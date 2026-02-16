use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration, Utc};
use ciam_database::Database;
use ciam_models::{
    oauth2::{
        AuthorizationCode, AuthorizationRequest, CreateAuthorizationCode,
        CreateAuthorizationRequest, CreateLoginSession, CreateRefreshToken, Jwk, Jwks,
        LoginSession, OAuthConsent, OidcDiscovery, RefreshToken, SigningKey, TokenError,
        TokenResponse, UserInfoResponse,
    },
    Application, User,
};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rand::{distributions::Alphanumeric, Rng};
use rsa::{pkcs8::DecodePublicKey, traits::PublicKeyParts, RsaPublicKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use crate::error::{AuthError, Result};

/// JWT Claims for access tokens
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub iss: String,           // Issuer
    pub sub: String,           // Subject (user ID)
    pub aud: Vec<String>,      // Audience (client IDs or API identifiers)
    pub exp: i64,              // Expiration time
    pub iat: i64,              // Issued at
    pub nbf: i64,              // Not before
    pub jti: String,           // JWT ID
    pub azp: String,           // Authorized party (client_id)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>, // Granted scopes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>, // Organization ID
}

/// JWT Claims for ID tokens (OIDC)
#[derive(Debug, Serialize, Deserialize)]
pub struct IdTokenClaims {
    pub iss: String,
    pub sub: String,
    pub aud: String,           // Single audience for id_token
    pub exp: i64,
    pub iat: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acr: Option<String>,   // Authentication Context Class Reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amr: Option<Vec<String>>, // Authentication Methods References
    #[serde(skip_serializing_if = "Option::is_none")]
    pub azp: Option<String>,
    // Standard OIDC claims
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_name: Option<String>,
}

pub struct OAuth2Service {
    db: Arc<Database>,
    issuer: String,
    signing_key: Option<SigningKey>,
}

impl OAuth2Service {
    pub async fn new(db: Arc<Database>, issuer: String) -> Result<Self> {
        // Load current signing key
        let signing_key = Self::load_current_signing_key(&db).await?;

        Ok(Self {
            db,
            issuer,
            signing_key,
        })
    }

    async fn load_current_signing_key(db: &Database) -> Result<Option<SigningKey>> {
        let key = sqlx::query_as::<_, SigningKey>(
            "SELECT * FROM signing_keys WHERE is_current = true LIMIT 1",
        )
        .fetch_optional(db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok(key)
    }

    // ========================================================================
    // OIDC DISCOVERY
    // ========================================================================

    pub fn get_discovery(&self) -> OidcDiscovery {
        OidcDiscovery::new(&self.issuer)
    }

    pub async fn get_jwks(&self) -> Result<Jwks> {
        let keys = sqlx::query_as::<_, SigningKey>(
            "SELECT * FROM signing_keys WHERE rotated_at IS NULL OR rotated_at > NOW() - INTERVAL '7 days'",
        )
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

        let jwks: Vec<Jwk> = keys
            .into_iter()
            .filter_map(|key| self.signing_key_to_jwk(&key).ok())
            .collect();

        Ok(Jwks { keys: jwks })
    }

    fn signing_key_to_jwk(&self, key: &SigningKey) -> Result<Jwk> {
        // Parse the PEM public key to extract n and e
        let public_key = RsaPublicKey::from_public_key_pem(&key.public_key_pem)
            .map_err(|e| AuthError::Internal(format!("Failed to parse public key: {}", e)))?;

        let n = URL_SAFE_NO_PAD.encode(public_key.n().to_bytes_be());
        let e = URL_SAFE_NO_PAD.encode(public_key.e().to_bytes_be());

        Ok(Jwk {
            kty: "RSA".to_string(),
            r#use: "sig".to_string(),
            kid: key.id.clone(),
            alg: key.algorithm.clone(),
            n,
            e,
        })
    }

    // ========================================================================
    // AUTHORIZATION REQUEST
    // ========================================================================

    pub async fn create_authorization_request(
        &self,
        request: CreateAuthorizationRequest,
    ) -> Result<AuthorizationRequest> {
        let request_id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let expires_at = Utc::now() + Duration::minutes(10);

        let auth_request = sqlx::query_as::<_, AuthorizationRequest>(
            r#"
            INSERT INTO oauth_authorization_requests (
                request_id, client_id, redirect_uri, response_type, scope, state,
                code_challenge, code_challenge_method, nonce, tenant_id,
                connection_hint, login_hint, prompt, max_age, ui_locales, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING *
            "#,
        )
        .bind(&request_id)
        .bind(&request.client_id)
        .bind(&request.redirect_uri)
        .bind(&request.response_type)
        .bind(&request.scope)
        .bind(&request.state)
        .bind(&request.code_challenge)
        .bind(&request.code_challenge_method)
        .bind(&request.nonce)
        .bind(request.organization_id)
        .bind(&request.connection_hint)
        .bind(&request.login_hint)
        .bind(&request.prompt)
        .bind(request.max_age)
        .bind(&request.ui_locales)
        .bind(expires_at)
        .fetch_one(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok(auth_request)
    }

    pub async fn get_authorization_request(&self, request_id: &str) -> Result<AuthorizationRequest> {
        sqlx::query_as::<_, AuthorizationRequest>(
            "SELECT * FROM oauth_authorization_requests WHERE request_id = $1 AND expires_at > NOW()",
        )
        .bind(request_id)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?
        .ok_or_else(|| AuthError::NotFound("Authorization request not found or expired".into()))
    }

    pub async fn delete_authorization_request(&self, request_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM oauth_authorization_requests WHERE request_id = $1")
            .bind(request_id)
            .execute(self.db.pool())
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok(())
    }

    // ========================================================================
    // AUTHORIZATION CODE
    // ========================================================================

    pub async fn create_authorization_code(
        &self,
        request: CreateAuthorizationCode,
    ) -> Result<String> {
        let code: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(48)
            .map(char::from)
            .collect();

        let expires_at = Utc::now() + Duration::minutes(10);

        sqlx::query(
            r#"
            INSERT INTO oauth_authorization_codes (
                code, client_id, user_id, tenant_id, redirect_uri, scope,
                code_challenge, code_challenge_method, nonce, state, response_type, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(&code)
        .bind(&request.client_id)
        .bind(request.user_id)
        .bind(request.organization_id)
        .bind(&request.redirect_uri)
        .bind(&request.scope)
        .bind(&request.code_challenge)
        .bind(&request.code_challenge_method)
        .bind(&request.nonce)
        .bind(&request.state)
        .bind(&request.response_type)
        .bind(expires_at)
        .execute(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

        info!(client_id = %request.client_id, user_id = %request.user_id, "Created authorization code");

        Ok(code)
    }

    pub async fn exchange_authorization_code(
        &self,
        code: &str,
        client_id: &str,
        redirect_uri: &str,
        code_verifier: Option<&str>,
    ) -> Result<(AuthorizationCode, User)> {
        // Get and validate the authorization code
        let auth_code = sqlx::query_as::<_, AuthorizationCode>(
            r#"
            SELECT * FROM oauth_authorization_codes
            WHERE code = $1 AND client_id = $2 AND expires_at > NOW() AND used_at IS NULL
            "#,
        )
        .bind(code)
        .bind(client_id)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?
        .ok_or_else(|| AuthError::BadRequest("Invalid or expired authorization code".into()))?;

        // Verify redirect_uri matches
        if auth_code.redirect_uri != redirect_uri {
            return Err(AuthError::BadRequest("redirect_uri mismatch".into()));
        }

        // Verify PKCE if code_challenge was provided
        if let Some(challenge) = &auth_code.code_challenge {
            let verifier = code_verifier.ok_or_else(|| {
                AuthError::BadRequest("code_verifier required for PKCE".into())
            })?;

            let method = auth_code
                .code_challenge_method
                .as_deref()
                .unwrap_or("plain");

            let computed_challenge = if method == "S256" {
                let mut hasher = Sha256::new();
                hasher.update(verifier.as_bytes());
                URL_SAFE_NO_PAD.encode(hasher.finalize())
            } else {
                verifier.to_string()
            };

            if &computed_challenge != challenge {
                return Err(AuthError::BadRequest("Invalid code_verifier".into()));
            }
        }

        // Mark code as used
        sqlx::query("UPDATE oauth_authorization_codes SET used_at = NOW() WHERE code = $1")
            .bind(code)
            .execute(self.db.pool())
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;

        // Get the user
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(auth_code.user_id)
            .fetch_one(self.db.pool())
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok((auth_code, user))
    }

    // ========================================================================
    // TOKEN GENERATION
    // ========================================================================

    pub async fn generate_tokens(
        &self,
        user: &User,
        application: &Application,
        scope: Option<&str>,
        nonce: Option<&str>,
        organization_id: Option<Uuid>,
        include_refresh_token: bool,
    ) -> Result<TokenResponse> {
        let signing_key = self
            .signing_key
            .as_ref()
            .ok_or_else(|| AuthError::Internal("No signing key configured".into()))?;

        let now = Utc::now();
        let jti = Uuid::new_v4().to_string();

        // Access Token
        let access_token_exp = now + Duration::seconds(application.access_token_lifetime_seconds as i64);
        let access_claims = AccessTokenClaims {
            iss: self.issuer.clone(),
            sub: user.id.to_string(),
            aud: vec![application.client_id.clone()],
            exp: access_token_exp.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            jti: jti.clone(),
            azp: application.client_id.clone(),
            scope: scope.map(String::from),
            org_id: organization_id.map(|id| id.to_string()),
        };

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(signing_key.id.clone());

        let encoding_key = EncodingKey::from_rsa_pem(signing_key.private_key_pem.as_bytes())
            .map_err(|e| AuthError::Internal(format!("Invalid signing key: {}", e)))?;

        let access_token = encode(&header, &access_claims, &encoding_key)
            .map_err(|e| AuthError::Internal(format!("Failed to encode access token: {}", e)))?;

        // ID Token (if openid scope requested)
        let id_token = if scope.map(|s| s.contains("openid")).unwrap_or(false) {
            // Use access_token_lifetime_seconds for id_token as well (typically same)
            let id_token_exp = now + Duration::seconds(application.access_token_lifetime_seconds as i64);

            // Parse scopes to determine which claims to include
            let scopes: Vec<&str> = scope.unwrap_or("").split_whitespace().collect();

            // Build full name from metadata
            let full_name = match (&user.metadata.first_name, &user.metadata.last_name) {
                (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
                (Some(first), None) => Some(first.clone()),
                (None, Some(last)) => Some(last.clone()),
                (None, None) => None,
            };

            let id_claims = IdTokenClaims {
                iss: self.issuer.clone(),
                sub: user.id.to_string(),
                aud: application.client_id.clone(),
                exp: id_token_exp.timestamp(),
                iat: now.timestamp(),
                auth_time: Some(now.timestamp()),
                nonce: nonce.map(String::from),
                acr: None,
                amr: Some(vec!["pwd".to_string()]),
                azp: Some(application.client_id.clone()),
                // Include email claims if 'email' scope
                email: if scopes.contains(&"email") {
                    Some(user.email.clone())
                } else {
                    None
                },
                email_verified: if scopes.contains(&"email") {
                    Some(user.email_verified)
                } else {
                    None
                },
                // Include profile claims if 'profile' scope
                name: if scopes.contains(&"profile") {
                    full_name
                } else {
                    None
                },
                given_name: if scopes.contains(&"profile") {
                    user.metadata.first_name.clone()
                } else {
                    None
                },
                family_name: if scopes.contains(&"profile") {
                    user.metadata.last_name.clone()
                } else {
                    None
                },
                picture: if scopes.contains(&"profile") {
                    user.metadata.avatar_url.clone()
                } else {
                    None
                },
                locale: None,
                updated_at: if scopes.contains(&"profile") {
                    Some(user.updated_at.timestamp())
                } else {
                    None
                },
                org_id: organization_id.map(|id| id.to_string()),
                org_name: None, // Could fetch from organization
            };

            Some(
                encode(&header, &id_claims, &encoding_key)
                    .map_err(|e| AuthError::Internal(format!("Failed to encode id token: {}", e)))?,
            )
        } else {
            None
        };

        // Refresh Token (if offline_access scope or configured)
        let refresh_token = if include_refresh_token
            && scope.map(|s| s.contains("offline_access")).unwrap_or(true)
        {
            Some(
                self.create_refresh_token(CreateRefreshToken {
                    client_id: application.client_id.clone(),
                    user_id: user.id,
                    organization_id,
                    scope: scope.map(String::from),
                    audience: None,
                    expires_in_seconds: Some(application.refresh_token_lifetime_seconds as i64),
                })
                .await?,
            )
        } else {
            None
        };

        // Record access token for introspection
        sqlx::query(
            r#"
            INSERT INTO oauth_access_tokens (jti, client_id, user_id, tenant_id, scope, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&jti)
        .bind(&application.client_id)
        .bind(user.id)
        .bind(organization_id)
        .bind(scope)
        .bind(access_token_exp)
        .execute(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: application.access_token_lifetime_seconds as i64,
            refresh_token,
            id_token,
            scope: scope.map(String::from),
        })
    }

    // ========================================================================
    // REFRESH TOKEN
    // ========================================================================

    async fn create_refresh_token(&self, request: CreateRefreshToken) -> Result<String> {
        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        let token_hash = {
            let mut hasher = Sha256::new();
            hasher.update(token.as_bytes());
            format!("{:x}", hasher.finalize())
        };

        let expires_at = request
            .expires_in_seconds
            .map(|secs| Utc::now() + Duration::seconds(secs));

        sqlx::query(
            r#"
            INSERT INTO oauth_refresh_tokens (
                token_hash, client_id, user_id, tenant_id, scope, audience, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(&token_hash)
        .bind(&request.client_id)
        .bind(request.user_id)
        .bind(request.organization_id)
        .bind(&request.scope)
        .bind(&request.audience)
        .bind(expires_at)
        .execute(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok(token)
    }

    pub async fn refresh_tokens(
        &self,
        refresh_token: &str,
        client_id: &str,
        application: &Application,
    ) -> Result<TokenResponse> {
        let token_hash = {
            let mut hasher = Sha256::new();
            hasher.update(refresh_token.as_bytes());
            format!("{:x}", hasher.finalize())
        };

        // Get and validate refresh token
        let stored_token = sqlx::query_as::<_, RefreshToken>(
            r#"
            SELECT * FROM oauth_refresh_tokens
            WHERE token_hash = $1 AND client_id = $2
              AND revoked_at IS NULL
              AND (expires_at IS NULL OR expires_at > NOW())
            "#,
        )
        .bind(&token_hash)
        .bind(client_id)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?
        .ok_or_else(|| AuthError::BadRequest("Invalid or expired refresh token".into()))?;

        // Update last_used_at
        sqlx::query("UPDATE oauth_refresh_tokens SET last_used_at = NOW() WHERE id = $1")
            .bind(stored_token.id)
            .execute(self.db.pool())
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;

        // Get user
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(stored_token.user_id)
            .fetch_one(self.db.pool())
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;

        // Generate new tokens (without creating a new refresh token - we reuse the existing one)
        let signing_key = self
            .signing_key
            .as_ref()
            .ok_or_else(|| AuthError::Internal("No signing key configured".into()))?;

        let now = Utc::now();
        let jti = Uuid::new_v4().to_string();

        let access_token_exp = now + Duration::seconds(application.access_token_lifetime_seconds as i64);
        let access_claims = AccessTokenClaims {
            iss: self.issuer.clone(),
            sub: user.id.to_string(),
            aud: vec![application.client_id.clone()],
            exp: access_token_exp.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            jti: jti.clone(),
            azp: application.client_id.clone(),
            scope: stored_token.scope.clone(),
            org_id: stored_token.organization_id.map(|id| id.to_string()),
        };

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(signing_key.id.clone());

        let encoding_key = EncodingKey::from_rsa_pem(signing_key.private_key_pem.as_bytes())
            .map_err(|e| AuthError::Internal(format!("Invalid signing key: {}", e)))?;

        let access_token = encode(&header, &access_claims, &encoding_key)
            .map_err(|e| AuthError::Internal(format!("Failed to encode access token: {}", e)))?;

        // Record access token
        sqlx::query(
            r#"
            INSERT INTO oauth_access_tokens (jti, client_id, user_id, tenant_id, scope, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&jti)
        .bind(client_id)
        .bind(user.id)
        .bind(stored_token.organization_id)
        .bind(&stored_token.scope)
        .bind(access_token_exp)
        .execute(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: application.access_token_lifetime_seconds as i64,
            refresh_token: None, // Don't issue new refresh token
            id_token: None,      // No id_token on refresh
            scope: stored_token.scope,
        })
    }

    pub async fn revoke_token(&self, token: &str, client_id: &str) -> Result<()> {
        let token_hash = {
            let mut hasher = Sha256::new();
            hasher.update(token.as_bytes());
            format!("{:x}", hasher.finalize())
        };

        // Try to revoke as refresh token
        let result = sqlx::query(
            "UPDATE oauth_refresh_tokens SET revoked_at = NOW() WHERE token_hash = $1 AND client_id = $2",
        )
        .bind(&token_hash)
        .bind(client_id)
        .execute(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            // Try to revoke as access token (by JTI - would need to decode the token)
            // For now, we just succeed silently (per RFC 7009)
        }

        Ok(())
    }

    // ========================================================================
    // USER INFO
    // ========================================================================

    pub async fn get_user_info(
        &self,
        user_id: Uuid,
        scope: Option<&str>,
        organization_id: Option<Uuid>,
    ) -> Result<UserInfoResponse> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(self.db.pool())
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;

        let scopes: Vec<&str> = scope.unwrap_or("openid").split_whitespace().collect();

        // Get organization name if org_id provided
        let org_name = if let Some(org_id) = organization_id {
            sqlx::query_scalar::<_, String>("SELECT name FROM tenants WHERE id = $1")
                .bind(org_id)
                .fetch_optional(self.db.pool())
                .await
                .map_err(|e| AuthError::Database(e.to_string()))?
        } else {
            None
        };

        // Build full name from metadata
        let full_name = match (&user.metadata.first_name, &user.metadata.last_name) {
            (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
            (Some(first), None) => Some(first.clone()),
            (None, Some(last)) => Some(last.clone()),
            (None, None) => None,
        };

        Ok(UserInfoResponse {
            sub: user.id.to_string(),
            name: if scopes.contains(&"profile") {
                full_name
            } else {
                None
            },
            given_name: if scopes.contains(&"profile") {
                user.metadata.first_name.clone()
            } else {
                None
            },
            family_name: if scopes.contains(&"profile") {
                user.metadata.last_name.clone()
            } else {
                None
            },
            email: if scopes.contains(&"email") {
                Some(user.email)
            } else {
                None
            },
            email_verified: if scopes.contains(&"email") {
                Some(user.email_verified)
            } else {
                None
            },
            picture: if scopes.contains(&"profile") {
                user.metadata.avatar_url.clone()
            } else {
                None
            },
            locale: None,
            updated_at: if scopes.contains(&"profile") {
                Some(user.updated_at.timestamp())
            } else {
                None
            },
            org_id: organization_id.map(|id| id.to_string()),
            org_name,
        })
    }

    // ========================================================================
    // LOGIN SESSION
    // ========================================================================

    pub async fn create_login_session(&self, request: CreateLoginSession) -> Result<String> {
        let session_token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        let token_hash = {
            let mut hasher = Sha256::new();
            hasher.update(session_token.as_bytes());
            format!("{:x}", hasher.finalize())
        };

        let expires_at = Utc::now() + Duration::seconds(request.expires_in_seconds);

        sqlx::query(
            r#"
            INSERT INTO login_sessions (
                session_token_hash, user_id, tenant_id, ip_address, user_agent,
                mfa_verified, expires_at
            ) VALUES ($1, $2, $3, $4::inet, $5, $6, $7)
            "#,
        )
        .bind(&token_hash)
        .bind(request.user_id)
        .bind(request.organization_id)
        .bind(&request.ip_address)
        .bind(&request.user_agent)
        .bind(request.mfa_verified)
        .bind(expires_at)
        .execute(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok(session_token)
    }

    pub async fn validate_login_session(&self, session_token: &str) -> Result<LoginSession> {
        let token_hash = {
            let mut hasher = Sha256::new();
            hasher.update(session_token.as_bytes());
            format!("{:x}", hasher.finalize())
        };

        let session = sqlx::query_as::<_, LoginSession>(
            r#"
            SELECT id, session_token_hash, user_id, tenant_id,
                   ip_address::TEXT as ip_address, user_agent,
                   authenticated_at, last_active_at, expires_at,
                   mfa_verified, mfa_verified_at, revoked_at, created_at
            FROM login_sessions
            WHERE session_token_hash = $1
              AND expires_at > NOW()
              AND revoked_at IS NULL
            "#,
        )
        .bind(&token_hash)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?
        .ok_or_else(|| AuthError::BadRequest("Invalid or expired session".into()))?;

        // Update last_active_at
        sqlx::query("UPDATE login_sessions SET last_active_at = NOW() WHERE id = $1")
            .bind(session.id)
            .execute(self.db.pool())
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok(session)
    }

    pub async fn revoke_login_session(&self, session_token: &str) -> Result<()> {
        let token_hash = {
            let mut hasher = Sha256::new();
            hasher.update(session_token.as_bytes());
            format!("{:x}", hasher.finalize())
        };

        sqlx::query("UPDATE login_sessions SET revoked_at = NOW() WHERE session_token_hash = $1")
            .bind(&token_hash)
            .execute(self.db.pool())
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok(())
    }

    // ========================================================================
    // CONSENT
    // ========================================================================

    pub async fn get_consent(
        &self,
        user_id: Uuid,
        client_id: &str,
    ) -> Result<Option<OAuthConsent>> {
        sqlx::query_as::<_, OAuthConsent>(
            "SELECT * FROM oauth_consents WHERE user_id = $1 AND client_id = $2 AND revoked_at IS NULL",
        )
        .bind(user_id)
        .bind(client_id)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))
    }

    pub async fn grant_consent(
        &self,
        user_id: Uuid,
        client_id: &str,
        organization_id: Option<Uuid>,
        scopes: Vec<String>,
    ) -> Result<OAuthConsent> {
        let consent = sqlx::query_as::<_, OAuthConsent>(
            r#"
            INSERT INTO oauth_consents (user_id, client_id, tenant_id, scopes)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id, client_id)
            DO UPDATE SET scopes = $4, granted_at = NOW(), revoked_at = NULL
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(client_id)
        .bind(organization_id)
        .bind(&scopes)
        .fetch_one(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok(consent)
    }

    pub async fn revoke_consent(&self, user_id: Uuid, client_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE oauth_consents SET revoked_at = NOW() WHERE user_id = $1 AND client_id = $2",
        )
        .bind(user_id)
        .bind(client_id)
        .execute(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok(())
    }

    // ========================================================================
    // APPLICATION VALIDATION
    // ========================================================================

    pub async fn get_application_by_client_id(
        &self,
        client_id: &str,
    ) -> Result<Application> {
        sqlx::query_as::<_, Application>(
            "SELECT * FROM applications WHERE client_id = $1 AND is_active = true",
        )
        .bind(client_id)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?
        .ok_or_else(|| AuthError::NotFound("Application not found".into()))
    }

    /// Generate an access token for client credentials grant (machine-to-machine)
    /// This creates a token for the application itself, with no user context
    pub async fn generate_client_credentials_token(
        &self,
        application: &Application,
        scope: &str,
    ) -> Result<String> {
        let signing_key = self
            .signing_key
            .as_ref()
            .ok_or_else(|| AuthError::Internal("No signing key configured".into()))?;

        let now = Utc::now();
        let jti = Uuid::new_v4().to_string();
        let access_token_exp = now + Duration::seconds(application.access_token_lifetime_seconds as i64);

        // For client credentials, the subject is the client_id (no user)
        let access_claims = AccessTokenClaims {
            iss: self.issuer.clone(),
            sub: application.client_id.clone(), // Subject is the application itself
            aud: vec![application.client_id.clone()],
            exp: access_token_exp.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            jti: jti.clone(),
            azp: application.client_id.clone(),
            scope: if scope.is_empty() { None } else { Some(scope.to_string()) },
            org_id: application.organization_id.map(|id| id.to_string()),
        };

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(signing_key.id.clone());

        let encoding_key = EncodingKey::from_rsa_pem(signing_key.private_key_pem.as_bytes())
            .map_err(|e| AuthError::Internal(format!("Invalid signing key: {}", e)))?;

        let access_token = encode(&header, &access_claims, &encoding_key)
            .map_err(|e| AuthError::Internal(format!("Failed to encode access token: {}", e)))?;

        // Record access token for introspection (no user_id for client credentials)
        sqlx::query(
            r#"
            INSERT INTO oauth_access_tokens (jti, client_id, user_id, tenant_id, scope, expires_at)
            VALUES ($1, $2, NULL, $3, $4, $5)
            "#,
        )
        .bind(&jti)
        .bind(&application.client_id)
        .bind(application.organization_id)
        .bind(if scope.is_empty() { None } else { Some(scope) })
        .bind(access_token_exp)
        .execute(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

        info!(
            client_id = %application.client_id,
            scope = %scope,
            "Generated client credentials token"
        );

        Ok(access_token)
    }

    pub fn validate_redirect_uri(&self, application: &Application, redirect_uri: &str) -> bool {
        application
            .callback_urls
            .iter()
            .any(|url| url == redirect_uri)
    }

    pub fn validate_client_secret(
        &self,
        application: &Application,
        client_secret: &str,
    ) -> bool {
        application
            .client_secret
            .as_ref()
            .map(|hash| {
                // Client secrets are stored as bcrypt hashes
                bcrypt::verify(client_secret, hash).unwrap_or(false)
            })
            .unwrap_or(false)
    }
}
