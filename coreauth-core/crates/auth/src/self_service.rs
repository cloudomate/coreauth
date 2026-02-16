use std::sync::Arc;
use chrono::{Duration, Utc};
use uuid::Uuid;
use ciam_database::Database;
use ciam_cache::Cache;
use ciam_models::self_service::*;

use crate::oauth2_service::OAuth2Service;
use crate::password::PasswordHasher;

/// Self-service flow service — manages login and registration flows.
pub struct SelfServiceFlowService {
    db: Arc<Database>,
    cache: Arc<Cache>,
    oauth2_service: Arc<OAuth2Service>,
}

const FLOW_TTL_SECONDS: usize = 600; // 10 minutes

impl SelfServiceFlowService {
    pub fn new(
        db: Arc<Database>,
        cache: Arc<Cache>,
        oauth2_service: Arc<OAuth2Service>,
    ) -> Self {
        Self { db, cache, oauth2_service }
    }

    // ── Flow CRUD ───────────────────────────────────────────

    fn flow_key(flow_type: &FlowType, id: &Uuid) -> String {
        let t = match flow_type {
            FlowType::Login => "login",
            FlowType::Registration => "registration",
        };
        format!("self_service_flow:{}:{}", t, id)
    }

    async fn save_flow(&self, flow: &SelfServiceFlow) -> crate::Result<()> {
        let key = Self::flow_key(&flow.flow_type, &flow.id);
        self.cache.set(&key, flow, Some(FLOW_TTL_SECONDS)).await
            .map_err(|e| crate::AuthError::Internal(format!("Failed to save flow: {}", e)))
    }

    pub async fn get_flow(&self, flow_type: &FlowType, flow_id: Uuid) -> crate::Result<SelfServiceFlow> {
        let key = Self::flow_key(flow_type, &flow_id);
        self.cache.get::<SelfServiceFlow>(&key).await
            .map_err(|e| crate::AuthError::Internal(format!("Failed to get flow: {}", e)))?
            .ok_or_else(|| crate::AuthError::NotFound("Flow not found or expired".into()))
    }

    async fn delete_flow(&self, flow: &SelfServiceFlow) -> crate::Result<()> {
        let key = Self::flow_key(&flow.flow_type, &flow.id);
        self.cache.delete(&key).await
            .map_err(|e| crate::AuthError::Internal(format!("Failed to delete flow: {}", e)))
    }

    // ── Login Flow ──────────────────────────────────────────

    pub async fn create_login_flow(
        &self,
        delivery_method: DeliveryMethod,
        organization_id: Option<Uuid>,
        request_url: String,
        authorization_request_id: Option<String>,
    ) -> crate::Result<SelfServiceFlow> {
        let flow_id = Uuid::new_v4();
        let now = Utc::now();

        // Generate CSRF token for browser flows
        let csrf_token = if delivery_method == DeliveryMethod::Browser {
            Some(generate_csrf_token())
        } else {
            None
        };

        // Look up client_id if we have an authorization request
        let client_id = if let Some(ref req_id) = authorization_request_id {
            let row: Option<(String,)> = sqlx::query_as(
                "SELECT client_id FROM oauth_authorization_requests WHERE request_id = $1 AND expires_at > NOW()"
            )
            .bind(req_id)
            .fetch_optional(self.db.pool())
            .await
            .map_err(|e| crate::AuthError::Internal(format!("DB error: {}", e)))?;
            row.map(|(id,)| id)
        } else {
            None
        };

        // Build UI nodes
        let nodes = self.build_login_ui_nodes(organization_id, csrf_token.as_deref()).await;

        let action = format!("/self-service/login?flow={}", flow_id);
        let flow = SelfServiceFlow {
            id: flow_id,
            flow_type: FlowType::Login,
            delivery_method,
            state: FlowState::Active,
            request_url,
            issued_at: now,
            expires_at: now + Duration::seconds(FLOW_TTL_SECONDS as i64),
            authorization_request_id,
            client_id,
            organization_id,
            csrf_token,
            authenticated_user_id: None,
            authentication_methods: vec![],
            mfa_challenge_token: None,
            ui: FlowUi {
                action,
                method: "POST".into(),
                nodes,
                messages: vec![],
            },
        };

        self.save_flow(&flow).await?;
        Ok(flow)
    }

    pub async fn submit_login_flow(
        &self,
        flow_id: Uuid,
        submit: LoginFlowSubmit,
    ) -> crate::Result<FlowResponse> {
        let mut flow = self.get_flow(&FlowType::Login, flow_id).await?;

        // CSRF check for browser flows
        if flow.delivery_method == DeliveryMethod::Browser {
            if let Some(ref expected) = flow.csrf_token {
                let provided = submit.csrf_token.as_deref().unwrap_or("");
                if provided != expected {
                    return Ok(flow_error(&mut flow, message_ids::CSRF_MISMATCH, "CSRF token mismatch"));
                }
            }
        }

        match submit.method.as_str() {
            "password" => self.handle_password_login(&mut flow, &submit).await,
            "totp" => self.handle_mfa_submit(&mut flow, &submit).await,
            "oidc" => self.handle_oidc_login(&flow, &submit).await,
            _ => Ok(flow_error(&mut flow, message_ids::INTERNAL_ERROR, "Unsupported method")),
        }
    }

    async fn handle_password_login(
        &self,
        flow: &mut SelfServiceFlow,
        submit: &LoginFlowSubmit,
    ) -> crate::Result<FlowResponse> {
        tracing::info!(flow_id = %flow.id, auth_request_id = ?flow.authorization_request_id, "handle_password_login: processing login");
        let email = match &submit.identifier {
            Some(e) if !e.is_empty() => e.clone(),
            _ => return Ok(field_error(flow, "identifier", message_ids::FIELD_REQUIRED, "Email is required")),
        };
        let password = match &submit.password {
            Some(p) if !p.is_empty() => p.clone(),
            _ => return Ok(field_error(flow, "password", message_ids::FIELD_REQUIRED, "Password is required")),
        };

        // Find user by email (optionally scoped to organization)
        let user: Option<ciam_models::User> = if let Some(org_id) = flow.organization_id {
            sqlx::query_as(
                "SELECT u.* FROM users u
                 JOIN tenant_members om ON u.id = om.user_id
                 WHERE u.email = $1 AND om.tenant_id = $2 AND u.is_active = true"
            )
            .bind(&email)
            .bind(org_id)
            .fetch_optional(self.db.pool())
            .await
            .map_err(|e| crate::AuthError::Internal(format!("DB error: {}", e)))?
        } else {
            sqlx::query_as(
                "SELECT * FROM users WHERE email = $1 AND is_active = true"
            )
            .bind(&email)
            .fetch_optional(self.db.pool())
            .await
            .map_err(|e| crate::AuthError::Internal(format!("DB error: {}", e)))?
        };

        let user = match user {
            Some(u) => u,
            None => return Ok(flow_error(flow, message_ids::INVALID_CREDENTIALS, "Invalid email or password")),
        };

        // Verify password
        let password_hash = user.password_hash.as_deref().unwrap_or("");
        if !PasswordHasher::verify(&password, password_hash).unwrap_or(false) {
            return Ok(flow_error(flow, message_ids::INVALID_CREDENTIALS, "Invalid email or password"));
        }

        // Determine organization_id if not set (find user's tenant)
        let org_id = flow.organization_id.or(user.default_tenant_id);

        // Check if MFA is required
        let mfa_required = self.is_mfa_required(user.id, org_id).await;
        if mfa_required {
            // Create MFA challenge
            let challenge_token = Uuid::new_v4().to_string();
            let expires_at = Utc::now() + Duration::minutes(5);
            sqlx::query(
                "INSERT INTO mfa_challenges (user_id, challenge_token, expires_at) VALUES ($1, $2, $3)"
            )
            .bind(user.id)
            .bind(&challenge_token)
            .bind(expires_at)
            .execute(self.db.pool())
            .await
            .map_err(|e| crate::AuthError::Internal(format!("DB error: {}", e)))?;

            // Transition to RequiresMfa
            flow.state = FlowState::RequiresMfa;
            flow.authenticated_user_id = Some(user.id);
            flow.authentication_methods.push("password".into());
            flow.mfa_challenge_token = Some(challenge_token);
            flow.ui = FlowUi {
                action: format!("/self-service/login?flow={}", flow.id),
                method: "POST".into(),
                nodes: self.build_mfa_ui_nodes(flow.csrf_token.as_deref()),
                messages: vec![UiMessage {
                    id: 1060000,
                    text: "Enter the code from your authenticator app".into(),
                    message_type: "info".into(),
                    context: None,
                }],
            };
            self.save_flow(flow).await?;
            return Ok(FlowResponse {
                session: None,
                session_token: None,
                redirect_browser_to: None,
                flow: Some(flow.clone()),
            });
        }

        // No MFA — complete the flow
        tracing::info!(user_id = %user.id, org_id = ?org_id, "handle_password_login: password verified, completing flow");
        flow.authenticated_user_id = Some(user.id);
        flow.authentication_methods.push("password".into());
        self.complete_flow(flow, user.id, org_id).await
    }

    async fn handle_mfa_submit(
        &self,
        flow: &mut SelfServiceFlow,
        submit: &LoginFlowSubmit,
    ) -> crate::Result<FlowResponse> {
        if flow.state != FlowState::RequiresMfa {
            return Ok(flow_error(flow, message_ids::INTERNAL_ERROR, "MFA not required for this flow"));
        }

        let totp_code = match &submit.totp_code {
            Some(c) if !c.is_empty() => c.clone(),
            _ => return Ok(field_error(flow, "totp_code", message_ids::FIELD_REQUIRED, "Code is required")),
        };

        let user_id = flow.authenticated_user_id
            .ok_or_else(|| crate::AuthError::Internal("No authenticated user in MFA flow".into()))?;

        let challenge_token = flow.mfa_challenge_token.as_deref()
            .ok_or_else(|| crate::AuthError::Internal("No MFA challenge token".into()))?;

        // Validate challenge
        let challenge: Option<(Uuid, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
            "SELECT user_id, expires_at FROM mfa_challenges WHERE challenge_token = $1"
        )
        .bind(challenge_token)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| crate::AuthError::Internal(format!("DB error: {}", e)))?;

        let (challenge_user_id, expires_at) = match challenge {
            Some(c) => c,
            None => return Ok(flow_error(flow, message_ids::INVALID_TOTP_CODE, "Invalid or expired MFA challenge")),
        };

        if challenge_user_id != user_id || expires_at < Utc::now() {
            return Ok(flow_error(flow, message_ids::INVALID_TOTP_CODE, "Invalid or expired MFA challenge"));
        }

        // Get TOTP secret
        let totp_secret: Option<(String,)> = sqlx::query_as(
            "SELECT secret FROM mfa_methods WHERE user_id = $1 AND method_type = 'totp' AND verified = true"
        )
        .bind(user_id)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| crate::AuthError::Internal(format!("DB error: {}", e)))?;

        let secret = match totp_secret {
            Some((s,)) => s,
            None => return Ok(flow_error(flow, message_ids::INTERNAL_ERROR, "No TOTP method configured")),
        };

        // Verify code
        if !crate::mfa::verify_totp(&secret, &totp_code).unwrap_or(false) {
            return Ok(flow_error(flow, message_ids::INVALID_TOTP_CODE, "Invalid verification code"));
        }

        // Clean up challenge
        let _ = sqlx::query("DELETE FROM mfa_challenges WHERE challenge_token = $1")
            .bind(challenge_token)
            .execute(self.db.pool())
            .await;

        flow.authentication_methods.push("totp".into());
        let org_id = flow.organization_id;
        self.complete_flow(flow, user_id, org_id).await
    }

    async fn handle_oidc_login(
        &self,
        _flow: &SelfServiceFlow,
        submit: &LoginFlowSubmit,
    ) -> crate::Result<FlowResponse> {
        // Social login: return redirect URL to the social provider
        let connection_id = submit.connection_id
            .ok_or_else(|| crate::AuthError::BadRequest("connection_id is required for OIDC method".into()))?;

        // Build the social login redirect URL
        let redirect_url = format!(
            "/login/social/{}?request_id={}",
            connection_id,
            _flow.authorization_request_id.as_deref().unwrap_or("")
        );

        Ok(FlowResponse {
            session: None,
            session_token: None,
            redirect_browser_to: Some(redirect_url),
            flow: None,
        })
    }

    // ── Registration Flow ───────────────────────────────────

    pub async fn create_registration_flow(
        &self,
        delivery_method: DeliveryMethod,
        organization_id: Option<Uuid>,
        request_url: String,
        authorization_request_id: Option<String>,
    ) -> crate::Result<SelfServiceFlow> {
        let flow_id = Uuid::new_v4();
        let now = Utc::now();

        let csrf_token = if delivery_method == DeliveryMethod::Browser {
            Some(generate_csrf_token())
        } else {
            None
        };

        let client_id = if let Some(ref req_id) = authorization_request_id {
            let row: Option<(String,)> = sqlx::query_as(
                "SELECT client_id FROM oauth_authorization_requests WHERE request_id = $1 AND expires_at > NOW()"
            )
            .bind(req_id)
            .fetch_optional(self.db.pool())
            .await
            .map_err(|e| crate::AuthError::Internal(format!("DB error: {}", e)))?;
            row.map(|(id,)| id)
        } else {
            None
        };

        let nodes = self.build_registration_ui_nodes(organization_id, csrf_token.as_deref()).await;

        let action = format!("/self-service/registration?flow={}", flow_id);
        let flow = SelfServiceFlow {
            id: flow_id,
            flow_type: FlowType::Registration,
            delivery_method,
            state: FlowState::Active,
            request_url,
            issued_at: now,
            expires_at: now + Duration::seconds(FLOW_TTL_SECONDS as i64),
            authorization_request_id,
            client_id,
            organization_id,
            csrf_token,
            authenticated_user_id: None,
            authentication_methods: vec![],
            mfa_challenge_token: None,
            ui: FlowUi {
                action,
                method: "POST".into(),
                nodes,
                messages: vec![],
            },
        };

        self.save_flow(&flow).await?;
        Ok(flow)
    }

    pub async fn submit_registration_flow(
        &self,
        flow_id: Uuid,
        submit: RegistrationFlowSubmit,
    ) -> crate::Result<FlowResponse> {
        let mut flow = self.get_flow(&FlowType::Registration, flow_id).await?;

        // CSRF check
        if flow.delivery_method == DeliveryMethod::Browser {
            if let Some(ref expected) = flow.csrf_token {
                let provided = submit.csrf_token.as_deref().unwrap_or("");
                if provided != expected {
                    return Ok(flow_error(&mut flow, message_ids::CSRF_MISMATCH, "CSRF token mismatch"));
                }
            }
        }

        match submit.method.as_str() {
            "password" => self.handle_password_registration(&mut flow, &submit).await,
            "oidc" => {
                let connection_id = submit.connection_id
                    .ok_or_else(|| crate::AuthError::BadRequest("connection_id required".into()))?;
                let redirect_url = format!(
                    "/login/social/{}?request_id={}",
                    connection_id,
                    flow.authorization_request_id.as_deref().unwrap_or("")
                );
                Ok(FlowResponse {
                    session: None,
                    session_token: None,
                    redirect_browser_to: Some(redirect_url),
                    flow: None,
                })
            }
            _ => Ok(flow_error(&mut flow, message_ids::INTERNAL_ERROR, "Unsupported method")),
        }
    }

    async fn handle_password_registration(
        &self,
        flow: &mut SelfServiceFlow,
        submit: &RegistrationFlowSubmit,
    ) -> crate::Result<FlowResponse> {
        let email = match &submit.email {
            Some(e) if !e.is_empty() => e.clone(),
            _ => return Ok(field_error(flow, "email", message_ids::FIELD_REQUIRED, "Email is required")),
        };
        let password = match &submit.password {
            Some(p) if !p.is_empty() => p.clone(),
            _ => return Ok(field_error(flow, "password", message_ids::FIELD_REQUIRED, "Password is required")),
        };

        if password.len() < 8 {
            return Ok(field_error(flow, "password", message_ids::PASSWORD_TOO_SHORT, "Password must be at least 8 characters"));
        }

        // Check if email already exists
        let exists: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM users WHERE email = $1"
        )
        .bind(&email)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| crate::AuthError::Internal(format!("DB error: {}", e)))?;

        if exists.is_some() {
            return Ok(flow_error(flow, message_ids::EMAIL_ALREADY_EXISTS, "An account with this email already exists"));
        }

        // Hash password and create user
        let password_hash = PasswordHasher::hash(&password)?;
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        // Build metadata
        let metadata = if let Some(ref name) = submit.full_name {
            let parts: Vec<&str> = name.splitn(2, ' ').collect();
            let first = parts.first().unwrap_or(&"");
            let last = if parts.len() > 1 { parts[1] } else { "" };
            serde_json::json!({
                "full_name": name,
                "first_name": first,
                "last_name": last,
            })
        } else {
            serde_json::json!({})
        };

        sqlx::query(
            "INSERT INTO users (id, email, password_hash, metadata, is_active, email_verified, created_at, updated_at)
             VALUES ($1, $2, $3, $4, true, false, $5, $5)"
        )
        .bind(user_id)
        .bind(&email)
        .bind(&password_hash)
        .bind(&metadata)
        .bind(now)
        .execute(self.db.pool())
        .await
        .map_err(|e| crate::AuthError::Internal(format!("DB error creating user: {}", e)))?;

        // Add to organization if specified
        if let Some(org_id) = flow.organization_id {
            let _ = sqlx::query(
                "INSERT INTO tenant_members (id, tenant_id, user_id, role, joined_at)
                 VALUES ($1, $2, $3, 'member', $4)
                 ON CONFLICT DO NOTHING"
            )
            .bind(Uuid::new_v4())
            .bind(org_id)
            .bind(user_id)
            .bind(now)
            .execute(self.db.pool())
            .await;
        }

        flow.authenticated_user_id = Some(user_id);
        flow.authentication_methods.push("password".into());
        let org_id = flow.organization_id;
        self.complete_flow(flow, user_id, org_id).await
    }

    // ── Flow Completion ─────────────────────────────────────

    async fn complete_flow(
        &self,
        flow: &mut SelfServiceFlow,
        user_id: Uuid,
        organization_id: Option<Uuid>,
    ) -> crate::Result<FlowResponse> {
        let now = Utc::now();
        flow.state = FlowState::Completed;

        // If linked to OAuth2 authorization request, create auth code and redirect
        if let Some(ref request_id) = flow.authorization_request_id {
            tracing::info!(request_id = %request_id, "complete_flow: looking up authorization request");
            // Get the authorization request
            let auth_req: Option<ciam_models::AuthorizationRequest> = sqlx::query_as(
                "SELECT * FROM oauth_authorization_requests WHERE request_id = $1 AND expires_at > NOW()"
            )
            .bind(request_id)
            .fetch_optional(self.db.pool())
            .await
            .map_err(|e| crate::AuthError::Internal(format!("DB error: {}", e)))?;

            if auth_req.is_none() {
                tracing::warn!(request_id = %request_id, "complete_flow: authorization request NOT FOUND or expired, falling through to session");
            }

            if let Some(auth_req) = auth_req {
                tracing::info!(request_id = %request_id, client_id = %auth_req.client_id, "complete_flow: found auth request, creating auth code");
                let code = self.oauth2_service.create_authorization_code(
                    ciam_models::CreateAuthorizationCode {
                        client_id: auth_req.client_id.clone(),
                        user_id,
                        organization_id,
                        redirect_uri: auth_req.redirect_uri.clone(),
                        scope: auth_req.scope.clone(),
                        code_challenge: auth_req.code_challenge.clone(),
                        code_challenge_method: auth_req.code_challenge_method.clone(),
                        nonce: auth_req.nonce.clone(),
                        state: auth_req.state.clone(),
                        response_type: auth_req.response_type.clone(),
                    }
                ).await?;

                // Delete auth request
                let _ = sqlx::query("DELETE FROM oauth_authorization_requests WHERE request_id = $1")
                    .bind(request_id)
                    .execute(self.db.pool())
                    .await;

                let redirect_url = format!(
                    "{}?code={}&state={}",
                    auth_req.redirect_uri,
                    code,
                    auth_req.state.as_deref().unwrap_or("")
                );

                self.delete_flow(flow).await?;
                return Ok(FlowResponse {
                    session: None,
                    session_token: None,
                    redirect_browser_to: Some(redirect_url),
                    flow: None,
                });
            }
        }

        // No OAuth2 link — create session directly
        let session_token = self.oauth2_service.create_login_session(
            ciam_models::CreateLoginSession {
                user_id,
                organization_id,
                ip_address: None,
                user_agent: None,
                mfa_verified: flow.authentication_methods.contains(&"totp".to_string()),
                expires_in_seconds: 86400 * 7, // 7 days
            }
        ).await?;

        // Get user info for session response
        let user: ciam_models::User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(self.db.pool())
            .await
            .map_err(|e| crate::AuthError::Internal(format!("DB error: {}", e)))?;

        let auth_methods: Vec<AuthMethodRef> = flow.authentication_methods.iter()
            .map(|m| AuthMethodRef { method: m.clone(), completed_at: now })
            .collect();

        let session = SessionResponse {
            id: Uuid::new_v4(),
            identity: IdentityResponse {
                id: user.id,
                email: user.email,
                email_verified: user.email_verified,
                metadata: serde_json::to_value(&user.metadata).ok(),
                created_at: user.created_at,
                updated_at: user.updated_at,
            },
            authenticated_at: now,
            expires_at: now + Duration::days(7),
            authentication_methods: auth_methods,
        };

        self.delete_flow(flow).await?;

        let response = FlowResponse {
            session: Some(session),
            session_token: if flow.delivery_method == DeliveryMethod::Api {
                Some(session_token)
            } else {
                None
            },
            redirect_browser_to: None,
            flow: None,
        };

        Ok(response)
    }

    // ── UI Node Builders ────────────────────────────────────

    async fn build_login_ui_nodes(
        &self,
        organization_id: Option<Uuid>,
        csrf_token: Option<&str>,
    ) -> Vec<UiNode> {
        let mut nodes = Vec::new();

        // CSRF token (hidden)
        if let Some(csrf) = csrf_token {
            nodes.push(UiNode {
                node_type: "input".into(),
                group: "default".into(),
                attributes: UiNodeAttributes {
                    name: "csrf_token".into(),
                    input_type: "hidden".into(),
                    value: Some(serde_json::Value::String(csrf.to_string())),
                    required: true,
                    disabled: false,
                    pattern: None,
                    autocomplete: None,
                    maxlength: None,
                },
                messages: vec![],
                meta: UiNodeMeta { label: None, connection_id: None },
            });
        }

        // Email
        nodes.push(UiNode {
            node_type: "input".into(),
            group: "default".into(),
            attributes: UiNodeAttributes {
                name: "identifier".into(),
                input_type: "email".into(),
                value: None,
                required: true,
                disabled: false,
                pattern: None,
                autocomplete: Some("username".into()),
                maxlength: None,
            },
            messages: vec![],
            meta: UiNodeMeta {
                label: Some(UiLabel { text: "Email".into() }),
                connection_id: None,
            },
        });

        // Password
        nodes.push(UiNode {
            node_type: "input".into(),
            group: "password".into(),
            attributes: UiNodeAttributes {
                name: "password".into(),
                input_type: "password".into(),
                value: None,
                required: true,
                disabled: false,
                pattern: None,
                autocomplete: Some("current-password".into()),
                maxlength: None,
            },
            messages: vec![],
            meta: UiNodeMeta {
                label: Some(UiLabel { text: "Password".into() }),
                connection_id: None,
            },
        });

        // Method (hidden)
        nodes.push(UiNode {
            node_type: "input".into(),
            group: "password".into(),
            attributes: UiNodeAttributes {
                name: "method".into(),
                input_type: "hidden".into(),
                value: Some(serde_json::Value::String("password".into())),
                required: false,
                disabled: false,
                pattern: None,
                autocomplete: None,
                maxlength: None,
            },
            messages: vec![],
            meta: UiNodeMeta { label: None, connection_id: None },
        });

        // Submit button
        nodes.push(UiNode {
            node_type: "input".into(),
            group: "password".into(),
            attributes: UiNodeAttributes {
                name: "method".into(),
                input_type: "submit".into(),
                value: Some(serde_json::Value::String("password".into())),
                required: false,
                disabled: false,
                pattern: None,
                autocomplete: None,
                maxlength: None,
            },
            messages: vec![],
            meta: UiNodeMeta {
                label: Some(UiLabel { text: "Sign in".into() }),
                connection_id: None,
            },
        });

        // Social login buttons
        let social = self.get_social_connections(organization_id).await;
        for conn in social {
            nodes.push(UiNode {
                node_type: "input".into(),
                group: "oidc".into(),
                attributes: UiNodeAttributes {
                    name: "provider".into(),
                    input_type: "submit".into(),
                    value: Some(serde_json::Value::String(conn.provider_type.clone())),
                    required: false,
                    disabled: false,
                    pattern: None,
                    autocomplete: None,
                    maxlength: None,
                },
                messages: vec![],
                meta: UiNodeMeta {
                    label: Some(UiLabel { text: format!("Continue with {}", conn.name) }),
                    connection_id: Some(conn.id),
                },
            });
        }

        nodes
    }

    async fn build_registration_ui_nodes(
        &self,
        organization_id: Option<Uuid>,
        csrf_token: Option<&str>,
    ) -> Vec<UiNode> {
        let mut nodes = Vec::new();

        if let Some(csrf) = csrf_token {
            nodes.push(UiNode {
                node_type: "input".into(),
                group: "default".into(),
                attributes: UiNodeAttributes {
                    name: "csrf_token".into(),
                    input_type: "hidden".into(),
                    value: Some(serde_json::Value::String(csrf.to_string())),
                    required: true,
                    disabled: false,
                    pattern: None,
                    autocomplete: None,
                    maxlength: None,
                },
                messages: vec![],
                meta: UiNodeMeta { label: None, connection_id: None },
            });
        }

        // Full name
        nodes.push(UiNode {
            node_type: "input".into(),
            group: "profile".into(),
            attributes: UiNodeAttributes {
                name: "full_name".into(),
                input_type: "text".into(),
                value: None,
                required: false,
                disabled: false,
                pattern: None,
                autocomplete: Some("name".into()),
                maxlength: None,
            },
            messages: vec![],
            meta: UiNodeMeta {
                label: Some(UiLabel { text: "Full Name".into() }),
                connection_id: None,
            },
        });

        // Email
        nodes.push(UiNode {
            node_type: "input".into(),
            group: "password".into(),
            attributes: UiNodeAttributes {
                name: "email".into(),
                input_type: "email".into(),
                value: None,
                required: true,
                disabled: false,
                pattern: None,
                autocomplete: Some("email".into()),
                maxlength: None,
            },
            messages: vec![],
            meta: UiNodeMeta {
                label: Some(UiLabel { text: "Email".into() }),
                connection_id: None,
            },
        });

        // Password
        nodes.push(UiNode {
            node_type: "input".into(),
            group: "password".into(),
            attributes: UiNodeAttributes {
                name: "password".into(),
                input_type: "password".into(),
                value: None,
                required: true,
                disabled: false,
                pattern: None,
                autocomplete: Some("new-password".into()),
                maxlength: None,
            },
            messages: vec![],
            meta: UiNodeMeta {
                label: Some(UiLabel { text: "Password".into() }),
                connection_id: None,
            },
        });

        // Method (hidden)
        nodes.push(UiNode {
            node_type: "input".into(),
            group: "password".into(),
            attributes: UiNodeAttributes {
                name: "method".into(),
                input_type: "hidden".into(),
                value: Some(serde_json::Value::String("password".into())),
                required: false,
                disabled: false,
                pattern: None,
                autocomplete: None,
                maxlength: None,
            },
            messages: vec![],
            meta: UiNodeMeta { label: None, connection_id: None },
        });

        // Submit
        nodes.push(UiNode {
            node_type: "input".into(),
            group: "password".into(),
            attributes: UiNodeAttributes {
                name: "method".into(),
                input_type: "submit".into(),
                value: Some(serde_json::Value::String("password".into())),
                required: false,
                disabled: false,
                pattern: None,
                autocomplete: None,
                maxlength: None,
            },
            messages: vec![],
            meta: UiNodeMeta {
                label: Some(UiLabel { text: "Create account".into() }),
                connection_id: None,
            },
        });

        // Social login buttons
        let social = self.get_social_connections(organization_id).await;
        for conn in social {
            nodes.push(UiNode {
                node_type: "input".into(),
                group: "oidc".into(),
                attributes: UiNodeAttributes {
                    name: "provider".into(),
                    input_type: "submit".into(),
                    value: Some(serde_json::Value::String(conn.provider_type.clone())),
                    required: false,
                    disabled: false,
                    pattern: None,
                    autocomplete: None,
                    maxlength: None,
                },
                messages: vec![],
                meta: UiNodeMeta {
                    label: Some(UiLabel { text: format!("Continue with {}", conn.name) }),
                    connection_id: Some(conn.id),
                },
            });
        }

        nodes
    }

    fn build_mfa_ui_nodes(&self, csrf_token: Option<&str>) -> Vec<UiNode> {
        let mut nodes = Vec::new();

        if let Some(csrf) = csrf_token {
            nodes.push(UiNode {
                node_type: "input".into(),
                group: "default".into(),
                attributes: UiNodeAttributes {
                    name: "csrf_token".into(),
                    input_type: "hidden".into(),
                    value: Some(serde_json::Value::String(csrf.to_string())),
                    required: true,
                    disabled: false,
                    pattern: None,
                    autocomplete: None,
                    maxlength: None,
                },
                messages: vec![],
                meta: UiNodeMeta { label: None, connection_id: None },
            });
        }

        // Method (hidden)
        nodes.push(UiNode {
            node_type: "input".into(),
            group: "totp".into(),
            attributes: UiNodeAttributes {
                name: "method".into(),
                input_type: "hidden".into(),
                value: Some(serde_json::Value::String("totp".into())),
                required: false,
                disabled: false,
                pattern: None,
                autocomplete: None,
                maxlength: None,
            },
            messages: vec![],
            meta: UiNodeMeta { label: None, connection_id: None },
        });

        // TOTP code input
        nodes.push(UiNode {
            node_type: "input".into(),
            group: "totp".into(),
            attributes: UiNodeAttributes {
                name: "totp_code".into(),
                input_type: "text".into(),
                value: None,
                required: true,
                disabled: false,
                pattern: Some("[0-9]{6}".into()),
                autocomplete: Some("one-time-code".into()),
                maxlength: Some(6),
            },
            messages: vec![],
            meta: UiNodeMeta {
                label: Some(UiLabel { text: "Authenticator Code".into() }),
                connection_id: None,
            },
        });

        // Submit
        nodes.push(UiNode {
            node_type: "input".into(),
            group: "totp".into(),
            attributes: UiNodeAttributes {
                name: "method".into(),
                input_type: "submit".into(),
                value: Some(serde_json::Value::String("totp".into())),
                required: false,
                disabled: false,
                pattern: None,
                autocomplete: None,
                maxlength: None,
            },
            messages: vec![],
            meta: UiNodeMeta {
                label: Some(UiLabel { text: "Verify".into() }),
                connection_id: None,
            },
        });

        nodes
    }

    // ── Helpers ──────────────────────────────────────────────

    async fn is_mfa_required(&self, user_id: Uuid, organization_id: Option<Uuid>) -> bool {
        // Check if user has any verified MFA methods
        let has_mfa: Option<(i64,)> = sqlx::query_as(
            "SELECT COUNT(*) FROM mfa_methods WHERE user_id = $1 AND verified = true"
        )
        .bind(user_id)
        .fetch_optional(self.db.pool())
        .await
        .ok()
        .flatten();

        let user_has_mfa = has_mfa.map(|(c,)| c > 0).unwrap_or(false);
        if !user_has_mfa {
            return false;
        }

        // Check if org requires MFA
        if let Some(org_id) = organization_id {
            let settings: Option<(serde_json::Value,)> = sqlx::query_as(
                "SELECT settings FROM tenants WHERE id = $1"
            )
            .bind(org_id)
            .fetch_optional(self.db.pool())
            .await
            .ok()
            .flatten();

            if let Some((val,)) = settings {
                if let Ok(s) = serde_json::from_value::<ciam_models::OrganizationSettings>(val) {
                    if s.security.mfa_required {
                        return true;
                    }
                }
            }
        }

        // User has MFA configured, so always require it when logging in
        user_has_mfa
    }

    async fn get_social_connections(&self, organization_id: Option<Uuid>) -> Vec<SocialConnectionInfo> {
        let mut connections = Vec::new();

        // Platform-level social connections
        let platform: Vec<SocialConnectionInfo> = sqlx::query_as(
            "SELECT id, name, provider_type FROM connections WHERE type = 'social' AND scope = 'platform' AND is_enabled = true"
        )
        .fetch_all(self.db.pool())
        .await
        .unwrap_or_default();
        connections.extend(platform);

        // Org-level social connections
        if let Some(org_id) = organization_id {
            let org: Vec<SocialConnectionInfo> = sqlx::query_as(
                "SELECT id, name, provider_type FROM connections WHERE type = 'social' AND scope = 'organization' AND organization_id = $1 AND is_enabled = true"
            )
            .bind(org_id)
            .fetch_all(self.db.pool())
            .await
            .unwrap_or_default();
            connections.extend(org);
        }

        connections
    }
}

// ── Internal Types ──────────────────────────────────────────

#[derive(Debug, Clone, sqlx::FromRow)]
struct SocialConnectionInfo {
    id: Uuid,
    name: String,
    provider_type: String,
}

// ── Helper Functions ────────────────────────────────────────

fn generate_csrf_token() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    hex::encode(bytes)
}

fn flow_error(flow: &mut SelfServiceFlow, id: i64, text: &str) -> FlowResponse {
    flow.ui.messages = vec![UiMessage {
        id,
        text: text.into(),
        message_type: "error".into(),
        context: None,
    }];
    FlowResponse {
        session: None,
        session_token: None,
        redirect_browser_to: None,
        flow: Some(flow.clone()),
    }
}

fn field_error(flow: &mut SelfServiceFlow, field_name: &str, id: i64, text: &str) -> FlowResponse {
    // Add error message to the specific field's node
    for node in &mut flow.ui.nodes {
        if node.attributes.name == field_name {
            node.messages = vec![UiMessage {
                id,
                text: text.into(),
                message_type: "error".into(),
                context: None,
            }];
            break;
        }
    }
    FlowResponse {
        session: None,
        session_token: None,
        redirect_browser_to: None,
        flow: Some(flow.clone()),
    }
}
