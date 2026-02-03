pub mod templates;

use crate::error::{AuthError, Result};
use ciam_cache::Cache;
use ciam_database::Database;
use ciam_models::{
    oidc_provider::{ClaimMappings, IdTokenClaims, OAuthState, OidcProvider},
    user::{NewUser, User, UserMetadata},
};
use chrono::Utc;
use openidconnect::{
    core::{
        CoreAuthenticationFlow, CoreClient, CoreIdTokenClaims, CoreIdTokenVerifier,
        CoreProviderMetadata, CoreResponseType, CoreUserInfoClaims,
    },
    reqwest::async_http_client,
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, PkceCodeChallenge,
    RedirectUrl, Scope, TokenResponse,
};
use rand::Rng;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub use templates::{OidcProviderTemplate, get_provider_template, list_provider_templates, apply_template_values};

pub struct OidcService {
    db: Database,
    cache: Cache,
}

impl OidcService {
    pub fn new(db: Database, cache: Cache) -> Self {
        Self { db, cache }
    }

    /// Get authorization URL for OIDC provider (tenant-scoped)
    pub async fn get_authorization_url(
        &self,
        tenant_id: Uuid,
        provider_id: Uuid,
        redirect_uri: &str,
    ) -> Result<(String, String)> {
        let provider = self.get_provider(tenant_id, provider_id).await?;

        if !provider.is_active {
            return Err(AuthError::InvalidCredentials); // Provider is disabled
        }

        // Create OIDC client
        let client_id = ClientId::new(provider.client_id.clone());
        let client_secret = ClientSecret::new(provider.client_secret.clone());
        let issuer = IssuerUrl::new(provider.issuer.clone())
            .map_err(|e| AuthError::ExternalProviderError(e.to_string()))?;

        // Discover provider metadata
        let provider_metadata = CoreProviderMetadata::discover_async(issuer, async_http_client)
            .await
            .map_err(|e| AuthError::ExternalProviderError(e.to_string()))?;

        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            client_id,
            Some(client_secret),
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect_uri.to_string())
                .map_err(|e| AuthError::ExternalProviderError(e.to_string()))?,
        );

        // Generate PKCE challenge
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate state and nonce for CSRF protection
        let csrf_state = CsrfToken::new_random();
        let nonce = Nonce::new_random();

        // Build authorization URL
        let csrf_state_clone = csrf_state.clone();
        let nonce_clone = nonce.clone();
        let mut auth_request = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                move || csrf_state_clone.clone(),
                move || nonce_clone.clone(),
            )
            .set_pkce_challenge(pkce_challenge);

        // Add scopes
        for scope in &provider.scopes {
            auth_request = auth_request.add_scope(Scope::new(scope.clone()));
        }

        let (auth_url, csrf_token, _nonce) = auth_request.url();

        // Store state in cache for 10 minutes (CSRF protection)
        let oauth_state = OAuthState {
            state: csrf_token.secret().clone(),
            nonce: nonce.secret().clone(),
            provider_id,
            redirect_uri: redirect_uri.to_string(),
            created_at: Utc::now(),
        };

        let state_key = format!("oauth:state:{}", csrf_token.secret());
        self.cache
            .set(&state_key, &oauth_state, Some(600))
            .await?;

        // Store PKCE verifier
        let pkce_key = format!("oauth:pkce:{}", csrf_token.secret());
        self.cache
            .set(&pkce_key, pkce_verifier.secret(), Some(600))
            .await
            ?;

        Ok((auth_url.to_string(), csrf_token.secret().clone()))
    }

    /// Handle OAuth callback and exchange code for tokens
    pub async fn handle_callback(
        &self,
        code: &str,
        state: &str,
        tenant_id: Uuid,
    ) -> Result<User> {
        // Verify state (CSRF protection)
        let state_key = format!("oauth:state:{}", state);
        let oauth_state: OAuthState = self
            .cache
            .get(&state_key)
            .await
            ?
            .ok_or(AuthError::InvalidCredentials)?;

        // Delete state to prevent replay attacks
        let _ = self.cache.delete(&state_key).await;

        // Get PKCE verifier
        let pkce_key = format!("oauth:pkce:{}", state);
        let pkce_verifier_secret: String = self
            .cache
            .get(&pkce_key)
            .await
            ?
            .ok_or(AuthError::InvalidCredentials)?;

        let _ = self.cache.delete(&pkce_key).await;

        // Get provider configuration (tenant-scoped)
        let provider = self.get_provider(tenant_id, oauth_state.provider_id).await?;

        // Create OIDC client
        let client_id = ClientId::new(provider.client_id.clone());
        let client_secret = ClientSecret::new(provider.client_secret.clone());
        let issuer = IssuerUrl::new(provider.issuer.clone())
            .map_err(|e| AuthError::ExternalProviderError(e.to_string()))?;

        let provider_metadata = CoreProviderMetadata::discover_async(issuer, async_http_client)
            .await
            .map_err(|e| AuthError::ExternalProviderError(e.to_string()))?;

        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            client_id,
            Some(client_secret),
        )
        .set_redirect_uri(
            RedirectUrl::new(oauth_state.redirect_uri.clone())
                .map_err(|e| AuthError::ExternalProviderError(e.to_string()))?,
        );

        // Exchange authorization code for tokens
        let pkce_verifier =
            openidconnect::PkceCodeVerifier::new(pkce_verifier_secret);

        let token_response = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
            .map_err(|e| AuthError::ExternalProviderError(e.to_string()))?;

        // Get ID token
        let id_token = token_response
            .id_token()
            .ok_or(AuthError::ExternalProviderError(
                "No ID token in response".to_string(),
            ))?;

        // Verify ID token
        let verifier: CoreIdTokenVerifier = client.id_token_verifier();
        let nonce = Nonce::new(oauth_state.nonce.clone());
        let id_token_claims: CoreIdTokenClaims = id_token
            .claims(&verifier, &nonce)
            .map_err(|e| AuthError::ExternalProviderError(e.to_string()))?
            .clone();

        // Extract user info from ID token
        let email = id_token_claims
            .email()
            .map(|e| e.as_str().to_string())
            .ok_or(AuthError::ExternalProviderError(
                "Email not found in ID token".to_string(),
            ))?;

        let provider_user_id = id_token_claims.subject().to_string();

        // Extract groups from ID token (using raw JWT payload)
        // Get the raw JWT string from the ID token by converting to string
        let jwt_string = format!("{}", id_token.to_string());
        let user_groups = self.extract_groups_from_jwt(&jwt_string, &provider)?;

        // Check if user already exists (linked to this provider)
        if let Some(existing_user) = self
            .find_user_by_provider(tenant_id, oauth_state.provider_id, &provider_user_id)
            .await.map_err(|e| AuthError::Internal(e.to_string()))?
        {
            // Sync groups for existing user
            self.sync_user_groups(existing_user.id, tenant_id, &user_groups, &provider).await?;
            return Ok(existing_user);
        }

        // Check if user exists by email
        let user_repo = ciam_database::repositories::users::UserRepository::new(self.db.pool().clone());
        if let Ok(existing_user) = user_repo.find_by_email(tenant_id, &email).await {
            // Link existing user to provider
            self.link_user_to_provider(
                existing_user.id,
                oauth_state.provider_id,
                &provider_user_id,
            )
            .await?;

            // Sync groups for existing user
            self.sync_user_groups(existing_user.id, tenant_id, &user_groups, &provider).await?;
            return Ok(existing_user);
        }

        // Create new user
        let metadata = self.extract_user_metadata(&id_token_claims, &provider.claim_mappings);
        let phone = None; // Extract from claims if available

        let new_user = NewUser {
            tenant_id: Some(tenant_id),
            email: email.clone(),
            password: Some(String::new()), // OAuth users don't have passwords
            phone,
            metadata: Some(metadata),
            is_platform_admin: false, // OIDC users are not platform admins
        };

        let user = user_repo.create(&new_user, "").await?; // Empty password for OAuth users

        // Link user to provider
        self.link_user_to_provider(user.id, oauth_state.provider_id, &provider_user_id)
            .await?;

        // Sync groups for new user
        self.sync_user_groups(user.id, tenant_id, &user_groups, &provider).await?;

        Ok(user)
    }

    async fn get_provider(&self, tenant_id: Uuid, provider_id: Uuid) -> Result<OidcProvider> {
        let provider = sqlx::query_as::<_, OidcProvider>(
            "SELECT * FROM oidc_providers WHERE id = $1 AND tenant_id = $2 AND is_active = true",
        )
        .bind(provider_id)
        .bind(tenant_id)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| AuthError::Internal(e.to_string()))?
        .ok_or(AuthError::InvalidCredentials)?;

        Ok(provider)
    }

    /// List OIDC providers for a tenant
    pub async fn list_providers(&self, tenant_id: Uuid) -> Result<Vec<OidcProvider>> {
        let providers = sqlx::query_as::<_, OidcProvider>(
            "SELECT * FROM oidc_providers WHERE tenant_id = $1 AND is_active = true ORDER BY name",
        )
        .bind(tenant_id)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| AuthError::Internal(e.to_string()))?;

        Ok(providers)
    }

    /// Create OIDC provider for a tenant (admin operation)
    pub async fn create_provider(
        &self,
        tenant_id: Uuid,
        name: String,
        provider_type: String,
        issuer: String,
        client_id: String,
        client_secret: String,
        authorization_endpoint: String,
        token_endpoint: String,
        userinfo_endpoint: Option<String>,
        jwks_uri: String,
        scopes: Vec<String>,
        claim_mappings: Option<ClaimMappings>,
        groups_claim: Option<String>,
        group_role_mappings: Option<HashMap<String, String>>,
    ) -> Result<OidcProvider> {
        let mappings = claim_mappings.unwrap_or_default();
        let groups_claim_value = groups_claim.unwrap_or_else(|| "groups".to_string());
        let group_mappings = group_role_mappings.unwrap_or_default();

        let provider = sqlx::query_as::<_, OidcProvider>(
            r#"INSERT INTO oidc_providers
               (tenant_id, name, provider_type, issuer, client_id, client_secret,
                authorization_endpoint, token_endpoint, userinfo_endpoint, jwks_uri, scopes, claim_mappings,
                groups_claim, group_role_mappings)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               RETURNING *"#,
        )
        .bind(tenant_id)
        .bind(name)
        .bind(provider_type)
        .bind(issuer)
        .bind(client_id)
        .bind(client_secret)
        .bind(authorization_endpoint)
        .bind(token_endpoint)
        .bind(userinfo_endpoint)
        .bind(jwks_uri)
        .bind(&scopes)
        .bind(sqlx::types::Json(&mappings))
        .bind(&groups_claim_value)
        .bind(sqlx::types::Json(&group_mappings))
        .fetch_one(self.db.pool())
        .await
        .map_err(|e| AuthError::Internal(e.to_string()))?;

        Ok(provider)
    }

    async fn find_user_by_provider(
        &self,
        tenant_id: Uuid,
        provider_id: Uuid,
        provider_user_id: &str,
    ) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE tenant_id = $1 AND provider_id = $2 AND provider_user_id = $3",
        )
        .bind(tenant_id)
        .bind(provider_id)
        .bind(provider_user_id)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| AuthError::Internal(e.to_string()))?;

        Ok(user)
    }

    async fn link_user_to_provider(
        &self,
        user_id: Uuid,
        provider_id: Uuid,
        provider_user_id: &str,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE users SET provider_id = $1, provider_user_id = $2 WHERE id = $3",
        )
        .bind(provider_id)
        .bind(provider_user_id)
        .bind(user_id)
        .execute(self.db.pool())
        .await
        .map_err(|e| AuthError::Internal(e.to_string()))?;

        Ok(())
    }

    fn extract_user_metadata(
        &self,
        claims: &CoreIdTokenClaims,
        _mappings: &ClaimMappings,
    ) -> UserMetadata {
        // For standard OIDC claims, use the built-in getters
        let first_name = claims.given_name().and_then(|n| n.get(None)).map(|s| s.as_str().to_string());
        let last_name = claims.family_name().and_then(|n| n.get(None)).map(|s| s.as_str().to_string());
        let avatar_url = claims.picture().and_then(|p| p.get(None)).map(|u| u.as_str().to_string());

        UserMetadata {
            first_name,
            last_name,
            avatar_url,
            language: None,
            timezone: None,
            custom: serde_json::Value::Null,
        }
    }

    /// Extract groups from ID token by parsing the raw JWT payload
    fn extract_groups_from_jwt(
        &self,
        jwt_str: &str,
        provider: &OidcProvider,
    ) -> Result<Vec<String>> {
        let groups_claim = provider.groups_claim.as_deref().unwrap_or("groups");

        // Parse JWT (it's in format: header.payload.signature)
        let parts: Vec<&str> = jwt_str.split('.').collect();
        if parts.len() != 3 {
            return Ok(Vec::new());
        }

        // Decode the payload (middle part)
        use base64::Engine;
        let payload_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|e| AuthError::Internal(format!("Failed to decode JWT payload: {}", e)))?;

        // Parse JSON
        let payload: Value = serde_json::from_slice(&payload_bytes)
            .map_err(|e| AuthError::Internal(format!("Failed to parse JWT payload: {}", e)))?;

        // Extract groups from the specified claim
        if let Some(groups_value) = payload.get(groups_claim) {
            match groups_value {
                Value::Array(arr) => {
                    Ok(arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect())
                }
                Value::String(s) => Ok(vec![s.clone()]),
                _ => Ok(Vec::new()),
            }
        } else {
            Ok(Vec::new())
        }
    }

    /// Synchronize user groups from OIDC provider to tenant roles
    async fn sync_user_groups(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        user_groups: &[String],
        provider: &OidcProvider,
    ) -> Result<()> {
        // Get group-to-role mappings from provider
        let group_mappings: HashMap<String, String> = provider
            .group_role_mappings
            .as_ref()
            .and_then(|json| serde_json::from_value(json.clone()).ok())
            .unwrap_or_default();

        if group_mappings.is_empty() {
            // No mappings configured, skip sync
            return Ok(());
        }

        // Find roles that should be assigned based on group mappings
        let mut role_names_to_assign: Vec<String> = Vec::new();
        for group in user_groups {
            if let Some(role_name) = group_mappings.get(group) {
                role_names_to_assign.push(role_name.clone());
            }
        }

        if role_names_to_assign.is_empty() {
            // No matching roles found
            return Ok(());
        }

        // Get role IDs from role names
        let role_ids: Vec<Uuid> = sqlx::query_scalar(
            r#"
            SELECT id FROM roles
            WHERE tenant_id = $1 AND name = ANY($2)
            "#,
        )
        .bind(tenant_id)
        .bind(&role_names_to_assign)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| AuthError::Internal(format!("Failed to fetch roles: {}", e)))?;

        // Remove all existing roles for this user (that were synced from OIDC)
        // We'll identify OIDC-synced roles by checking if they match any mapping
        let all_mapped_role_names: Vec<String> = group_mappings.values().cloned().collect();

        sqlx::query(
            r#"
            DELETE FROM user_roles
            WHERE user_id = $1
              AND role_id IN (
                SELECT id FROM roles
                WHERE tenant_id = $2 AND name = ANY($3)
              )
            "#,
        )
        .bind(user_id)
        .bind(tenant_id)
        .bind(&all_mapped_role_names)
        .execute(self.db.pool())
        .await
        .map_err(|e| AuthError::Internal(format!("Failed to remove old roles: {}", e)))?;

        // Assign new roles
        for role_id in &role_ids {
            sqlx::query(
                r#"
                INSERT INTO user_roles (user_id, role_id)
                VALUES ($1, $2)
                ON CONFLICT (user_id, role_id) DO NOTHING
                "#,
            )
            .bind(user_id)
            .bind(role_id)
            .execute(self.db.pool())
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to assign role: {}", e)))?;
        }

        tracing::info!(
            "Synced groups for user {}: assigned {} roles from {} groups",
            user_id,
            role_ids.len(),
            user_groups.len()
        );

        Ok(())
    }
}
