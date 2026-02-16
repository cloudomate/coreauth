use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use std::sync::Arc;

use crate::auth;
use crate::config::{AuthMode, RouteTarget, UnauthAction};
use crate::reverse_proxy::build_identity_headers;
use crate::route_matcher::{extract_object_id, match_route};
use crate::session::SessionManager;
use crate::ProxyState;

/// Main request handler — runs the auth + FGA + proxy pipeline.
pub async fn handle_request(
    state: Arc<ProxyState>,
    req: Request<Body>,
) -> Response<Body> {
    let method = req.method().as_str().to_string();
    let path = req.uri().path().to_string();
    let query_string = req.uri().query().map(|q| q.to_string());

    // ── Auth routes (handled directly, not proxied) ──────────────────
    if path.starts_with("/auth/") {
        return handle_auth_route(state, req, &path).await;
    }

    // ── Match route against config rules ─────────────────────────────
    let matched = match_route(&state.config.routes, &path, &method);

    let (rule, path_params) = match matched {
        Some((rule, params)) => (Some(rule.clone()), params),
        None => (None, std::collections::HashMap::new()),
    };

    let auth_mode = rule.as_ref().map(|r| &r.auth).unwrap_or(&AuthMode::Optional);
    let unauth_action = rule.as_ref()
        .map(|r| &r.on_unauthenticated)
        .unwrap_or(&UnauthAction::RedirectLogin);
    let target = rule.as_ref().map(|r| &r.target).unwrap_or(&RouteTarget::Upstream);
    let fga_rule = rule.as_ref().and_then(|r| r.fga.as_ref());

    // ── Session check ────────────────────────────────────────────────
    // Extract session ID from cookie (sync) before any async work
    let session_id = state.session_manager.extract_session_id(req.headers());
    let mut session = match &session_id {
        Some(id) => state.session_manager.get_session(id).await,
        None => None,
    };

    // Token refresh if session exists but token expired
    if let Some(ref mut sess) = session {
        if SessionManager::is_token_expired(sess) {
            if let Some(ref refresh_token) = sess.refresh_token {
                match crate::session::refresh_access_token(
                    &state.http_client,
                    &state.config.coreauth.url,
                    &state.config.coreauth.client_id,
                    &state.config.coreauth.client_secret,
                    refresh_token,
                ).await {
                    Ok(token_resp) => {
                        tracing::debug!("Token refreshed for user {}", sess.user_id);
                        sess.access_token = token_resp.access_token;
                        if let Some(new_refresh) = token_resp.refresh_token {
                            sess.refresh_token = Some(new_refresh);
                        }
                        if let Some(new_id_token) = token_resp.id_token {
                            sess.id_token = Some(new_id_token);
                        }
                        sess.expires_at = chrono::Utc::now().timestamp() + token_resp.expires_in;
                        // Update the server-side session store
                        if let Some(ref id) = session_id {
                            state.session_manager.update_session(id, sess.clone()).await;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Token refresh failed: {}", e);
                        // Clear invalid session
                        if let Some(ref id) = session_id {
                            state.session_manager.destroy_session(id).await;
                        }
                        session = None;
                    }
                }
            } else {
                // No refresh token — session expired
                if let Some(ref id) = session_id {
                    state.session_manager.destroy_session(id).await;
                }
                session = None;
            }
        }
    }

    // ── Auth enforcement ─────────────────────────────────────────────
    match auth_mode {
        AuthMode::Required => {
            if session.is_none() {
                return match unauth_action {
                    UnauthAction::Status401 => {
                        json_error(StatusCode::UNAUTHORIZED, "Authentication required")
                    }
                    UnauthAction::RedirectLogin => {
                        let login_url = format!("/auth/login?redirect={}", urlencoding::encode(&path));
                        Response::builder()
                            .status(StatusCode::FOUND)
                            .header("location", &login_url)
                            .body(Body::empty())
                            .unwrap()
                    }
                };
            }
        }
        AuthMode::None => {
            // No auth needed — proceed without session
        }
        AuthMode::Optional => {
            // Session optional — proceed either way
        }
    }

    // ── FGA permission check ─────────────────────────────────────────
    if let Some(fga_rule) = fga_rule {
        if let Some(ref sess) = session {
            let object_id = extract_object_id(
                &fga_rule.object_id,
                &path_params,
                query_string.as_deref(),
                req.headers(),
            );

            match object_id {
                Some(obj_id) => {
                    match state.fga_client.check_permission(
                        &sess.user_id,
                        &fga_rule.relation,
                        &fga_rule.object_type,
                        &obj_id,
                        &sess.access_token,
                    ).await {
                        Ok(true) => {
                            tracing::debug!(
                                "FGA allowed: user={} {}:{} relation={}",
                                sess.user_id, fga_rule.object_type, obj_id, fga_rule.relation
                            );
                        }
                        Ok(false) => {
                            tracing::info!(
                                "FGA denied: user={} {}:{} relation={}",
                                sess.user_id, fga_rule.object_type, obj_id, fga_rule.relation
                            );
                            return json_error(
                                StatusCode::FORBIDDEN,
                                "You don't have permission to perform this action",
                            );
                        }
                        Err(e) => {
                            tracing::error!("FGA check error: {}", e);
                            return json_error(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "Authorization check failed",
                            );
                        }
                    }
                }
                None => {
                    tracing::warn!(
                        "FGA object_id extraction failed for spec '{}' on path '{}'",
                        fga_rule.object_id, path
                    );
                    return json_error(
                        StatusCode::BAD_REQUEST,
                        "Could not determine resource for authorization",
                    );
                }
            }
        }
        // If no session but FGA rule exists, the auth: required check above
        // should have already rejected the request. If auth is optional
        // and user is not logged in, skip FGA (no user to check).
    }

    // ── Build identity headers ───────────────────────────────────────
    let identity_headers = session.as_ref().map(|s| build_identity_headers(s));

    // ── Forward request ─────────────────────────────────────────────
    let proxy = match target {
        RouteTarget::Coreauth => &state.coreauth_proxy,
        RouteTarget::Upstream => &state.reverse_proxy,
    };

    match proxy.forward(req, identity_headers).await {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!("Proxy forward error: {}", e);
            json_error(StatusCode::BAD_GATEWAY, "Upstream service unavailable")
        }
    }
}

/// Route /auth/* requests to the appropriate auth handler.
async fn handle_auth_route(
    state: Arc<ProxyState>,
    req: Request<Body>,
    path: &str,
) -> Response<Body> {
    match path {
        "/auth/login" => {
            let query: auth::LoginQuery = req.uri().query()
                .and_then(|q| serde_urlencoded::from_str(q).ok())
                .unwrap_or(auth::LoginQuery { email: None, redirect: None });
            auth::handle_login(state, query).await
        }
        "/auth/callback" => {
            let query: auth::CallbackQuery = req.uri().query()
                .and_then(|q| serde_urlencoded::from_str(q).ok())
                .unwrap_or(auth::CallbackQuery {
                    code: None,
                    state: None,
                    error: None,
                    error_description: None,
                });
            auth::handle_callback(state, query).await
        }
        "/auth/logout" => {
            let query: auth::LogoutQuery = req.uri().query()
                .and_then(|q| serde_urlencoded::from_str(q).ok())
                .unwrap_or(auth::LogoutQuery { redirect: None });
            auth::handle_logout(state, req.headers(), query).await
        }
        "/auth/userinfo" => {
            auth::handle_userinfo(state, req.headers()).await
        }
        _ => {
            json_error(StatusCode::NOT_FOUND, "Unknown auth endpoint")
        }
    }
}


fn json_error(status: StatusCode, message: &str) -> Response<Body> {
    let body = serde_json::json!({ "error": message });
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}
