use std::sync::Arc;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    Json,
};
use ciam_models::self_service::*;
use crate::handlers::ErrorResponse;
use crate::AppState;

// ── Login Flow ──────────────────────────────────────────────

/// GET /self-service/login/browser
/// Creates a login flow for browser apps (sets CSRF cookie).
pub async fn create_login_flow_browser(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CreateFlowQuery>,
    headers: HeaderMap,
) -> Response {
    let request_url = headers.get("referer")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("/self-service/login/browser")
        .to_string();

    match state.self_service_service.create_login_flow(
        DeliveryMethod::Browser,
        params.organization_id,
        request_url,
        params.request_id,
    ).await {
        Ok(flow) => {
            let csrf = flow.csrf_token.clone().unwrap_or_default();
            let clean = sanitize_flow(flow);
            let mut response = Json(&clean).into_response();
            // Set CSRF cookie
            if let Ok(cookie) = format!(
                "coreauth_csrf={}; Path=/self-service; HttpOnly; SameSite=Lax; Max-Age=600",
                csrf
            ).parse() {
                response.headers_mut().insert(header::SET_COOKIE, cookie);
            }
            response
        }
        Err(e) => error_response(e),
    }
}

/// GET /self-service/login/api
/// Creates a login flow for API/native apps (no CSRF).
pub async fn create_login_flow_api(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CreateFlowQuery>,
) -> Response {
    match state.self_service_service.create_login_flow(
        DeliveryMethod::Api,
        params.organization_id,
        "/self-service/login/api".into(),
        params.request_id,
    ).await {
        Ok(flow) => Json(&sanitize_flow(flow)).into_response(),
        Err(e) => error_response(e),
    }
}

/// GET /self-service/login?flow=<id>
/// Retrieves an existing login flow.
pub async fn get_login_flow(
    State(state): State<Arc<AppState>>,
    Query(params): Query<FlowQuery>,
) -> Response {
    match state.self_service_service.get_flow(&FlowType::Login, params.flow).await {
        Ok(flow) => Json(&sanitize_flow(flow)).into_response(),
        Err(e) => error_response(e),
    }
}

/// POST /self-service/login?flow=<id>
/// Submits credentials to a login flow.
pub async fn submit_login_flow(
    State(state): State<Arc<AppState>>,
    Query(params): Query<FlowQuery>,
    Json(submit): Json<LoginFlowSubmit>,
) -> Response {
    match state.self_service_service.submit_login_flow(params.flow, submit).await {
        Ok(resp) => flow_response(resp),
        Err(e) => error_response(e),
    }
}

// ── Registration Flow ───────────────────────────────────────

/// GET /self-service/registration/browser
pub async fn create_registration_flow_browser(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CreateFlowQuery>,
    headers: HeaderMap,
) -> Response {
    let request_url = headers.get("referer")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("/self-service/registration/browser")
        .to_string();

    match state.self_service_service.create_registration_flow(
        DeliveryMethod::Browser,
        params.organization_id,
        request_url,
        params.request_id,
    ).await {
        Ok(flow) => {
            let csrf = flow.csrf_token.clone().unwrap_or_default();
            let clean = sanitize_flow(flow);
            let mut response = Json(&clean).into_response();
            if let Ok(cookie) = format!(
                "coreauth_csrf={}; Path=/self-service; HttpOnly; SameSite=Lax; Max-Age=600",
                csrf
            ).parse() {
                response.headers_mut().insert(header::SET_COOKIE, cookie);
            }
            response
        }
        Err(e) => error_response(e),
    }
}

/// GET /self-service/registration/api
pub async fn create_registration_flow_api(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CreateFlowQuery>,
) -> Response {
    match state.self_service_service.create_registration_flow(
        DeliveryMethod::Api,
        params.organization_id,
        "/self-service/registration/api".into(),
        params.request_id,
    ).await {
        Ok(flow) => Json(&sanitize_flow(flow)).into_response(),
        Err(e) => error_response(e),
    }
}

/// GET /self-service/registration?flow=<id>
pub async fn get_registration_flow(
    State(state): State<Arc<AppState>>,
    Query(params): Query<FlowQuery>,
) -> Response {
    match state.self_service_service.get_flow(&FlowType::Registration, params.flow).await {
        Ok(flow) => Json(&sanitize_flow(flow)).into_response(),
        Err(e) => error_response(e),
    }
}

/// POST /self-service/registration?flow=<id>
pub async fn submit_registration_flow(
    State(state): State<Arc<AppState>>,
    Query(params): Query<FlowQuery>,
    Json(submit): Json<RegistrationFlowSubmit>,
) -> Response {
    match state.self_service_service.submit_registration_flow(params.flow, submit).await {
        Ok(resp) => flow_response(resp),
        Err(e) => error_response(e),
    }
}

// ── Session ─────────────────────────────────────────────────

/// GET /sessions/whoami
/// Returns the current session based on the session cookie or Authorization header.
pub async fn whoami(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    // Try session cookie first
    let session_token = headers.get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';')
                .find_map(|c| {
                    let c = c.trim();
                    if c.starts_with("coreauth_session=") {
                        Some(c.trim_start_matches("coreauth_session=").to_string())
                    } else {
                        None
                    }
                })
        })
        // Fall back to Bearer token
        .or_else(|| {
            headers.get(header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .map(|t| t.to_string())
        });

    let session_token = match session_token {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("unauthorized", "No session found")),
            ).into_response();
        }
    };

    match state.oauth2_service.validate_login_session(&session_token).await {
        Ok(session) => {
            // Get user info
            let user: Option<ciam_models::User> = sqlx::query_as("SELECT * FROM users WHERE id = $1")
                .bind(session.user_id)
                .fetch_optional(state.auth_service.db.pool())
                .await
                .ok()
                .flatten();

            match user {
                Some(user) => {
                    let now = chrono::Utc::now();
                    let resp = SessionResponse {
                        id: session.id,
                        identity: IdentityResponse {
                            id: user.id,
                            email: user.email,
                            email_verified: user.email_verified,
                            metadata: serde_json::to_value(&user.metadata).ok(),
                            created_at: user.created_at,
                            updated_at: user.updated_at,
                        },
                        authenticated_at: session.authenticated_at,
                        expires_at: session.expires_at,
                        authentication_methods: vec![AuthMethodRef {
                            method: "password".into(),
                            completed_at: now,
                        }],
                    };
                    Json(&resp).into_response()
                }
                None => (
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse::new("unauthorized", "User not found")),
                ).into_response(),
            }
        }
        Err(_) => (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("unauthorized", "Invalid or expired session")),
        ).into_response(),
    }
}

// ── Helpers ─────────────────────────────────────────────────

/// Strip internal fields (authenticated_user_id, mfa_challenge_token, etc.)
/// from the flow before sending to clients.
fn sanitize_flow(mut flow: SelfServiceFlow) -> SelfServiceFlow {
    flow.authenticated_user_id = None;
    flow.mfa_challenge_token = None;
    flow.authentication_methods = vec![];
    flow
}

fn sanitize_response(mut resp: FlowResponse) -> FlowResponse {
    resp.flow = resp.flow.map(sanitize_flow);
    resp
}

fn flow_response(resp: FlowResponse) -> Response {
    let resp = sanitize_response(resp);
    // If there's a session, the flow completed successfully
    if resp.session.is_some() || resp.redirect_browser_to.is_some() {
        Json(&resp).into_response()
    } else {
        // Flow returned with errors or intermediate state
        (StatusCode::BAD_REQUEST, Json(&resp)).into_response()
    }
}

fn error_response(err: ciam_auth::AuthError) -> Response {
    let (status, code, message) = match &err {
        ciam_auth::AuthError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg.as_str()),
        ciam_auth::AuthError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg.as_str()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "An internal error occurred"),
    };
    (status, Json(ErrorResponse::new(code, message))).into_response()
}
