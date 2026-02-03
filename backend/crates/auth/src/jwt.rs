use crate::error::{AuthError, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,                    // User ID
    pub tenant_id: Option<String>,      // Legacy: backward compatibility (deprecated)
    pub organization_id: Option<String>, // Organization ID (if user is in org context)
    pub organization_slug: Option<String>, // Organization slug for easy reference
    pub role: Option<String>,           // Org-scoped role (admin, member, viewer, etc.)
    pub is_platform_admin: bool,        // Platform admin flag (true for platform admins)
    pub email: String,                  // User email
    pub exp: i64,                       // Expiration time
    pub iat: i64,                       // Issued at
    pub jti: String,                    // JWT ID (unique identifier)
    pub token_type: TokenType,          // access or refresh
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
    Enrollment,
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    algorithm: Algorithm,
    access_token_exp_hours: i64,
    refresh_token_exp_days: i64,
}

impl JwtService {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            algorithm: Algorithm::HS256,
            access_token_exp_hours: 1,  // 1 hour default
            refresh_token_exp_days: 30, // 30 days default
        }
    }

    pub fn from_env() -> Self {
        let secret = std::env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set");

        let access_token_exp_hours = std::env::var("JWT_EXPIRATION_HOURS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        let refresh_token_exp_days = std::env::var("REFRESH_TOKEN_EXPIRATION_DAYS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            algorithm: Algorithm::HS256,
            access_token_exp_hours,
            refresh_token_exp_days,
        }
    }

    /// Generate an access token (hierarchical model)
    pub fn generate_access_token(
        &self,
        user_id: Uuid,
        email: &str,
        organization_id: Option<Uuid>,
        organization_slug: Option<String>,
        role: Option<String>,
        is_platform_admin: bool,
    ) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.access_token_exp_hours);

        let claims = Claims {
            sub: user_id.to_string(),
            tenant_id: organization_id.as_ref().map(|id| id.to_string()), // Legacy compatibility
            organization_id: organization_id.map(|id| id.to_string()),
            organization_slug,
            role,
            is_platform_admin,
            email: email.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            token_type: TokenType::Access,
        };

        let token = encode(&Header::new(self.algorithm), &claims, &self.encoding_key)?;
        Ok(token)
    }

    /// Generate an access token (legacy compatibility)
    /// Use this for backward compatibility with flat tenant model
    pub fn generate_access_token_legacy(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        email: &str,
    ) -> Result<String> {
        self.generate_access_token(
            user_id,
            email,
            Some(tenant_id),
            None,
            None,
            false,
        )
    }

    /// Generate a refresh token (hierarchical model)
    pub fn generate_refresh_token(
        &self,
        user_id: Uuid,
        email: &str,
        organization_id: Option<Uuid>,
        organization_slug: Option<String>,
        role: Option<String>,
        is_platform_admin: bool,
    ) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::days(self.refresh_token_exp_days);

        let claims = Claims {
            sub: user_id.to_string(),
            tenant_id: organization_id.as_ref().map(|id| id.to_string()), // Legacy compatibility
            organization_id: organization_id.map(|id| id.to_string()),
            organization_slug,
            role,
            is_platform_admin,
            email: email.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            token_type: TokenType::Refresh,
        };

        let token = encode(&Header::new(self.algorithm), &claims, &self.encoding_key)?;
        Ok(token)
    }

    /// Generate a refresh token (legacy compatibility)
    /// Use this for backward compatibility with flat tenant model
    pub fn generate_refresh_token_legacy(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        email: &str,
    ) -> Result<String> {
        self.generate_refresh_token(
            user_id,
            email,
            Some(tenant_id),
            None,
            None,
            false,
        )
    }

    /// Generate an enrollment token for MFA setup
    /// Short-lived (10 minutes), single-use token for MFA enrollment
    pub fn generate_enrollment_token(
        &self,
        user_id: Uuid,
        email: &str,
        organization_id: Option<Uuid>,
    ) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::minutes(10); // 10 minute expiration

        let claims = Claims {
            sub: user_id.to_string(),
            tenant_id: organization_id.as_ref().map(|id| id.to_string()),
            organization_id: organization_id.map(|id| id.to_string()),
            organization_slug: None,
            role: None,
            is_platform_admin: false,
            email: email.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            token_type: TokenType::Enrollment,
        };

        let token = encode(&Header::new(self.algorithm), &claims, &self.encoding_key)?;
        Ok(token)
    }

    /// Validate and decode a token
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let validation = Validation::new(self.algorithm);

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;

        Ok(token_data.claims)
    }

    /// Validate access token specifically
    pub fn validate_access_token(&self, token: &str) -> Result<Claims> {
        let claims = self.validate_token(token)?;

        if claims.token_type != TokenType::Access {
            return Err(AuthError::InvalidToken(
                "Token is not an access token".to_string(),
            ));
        }

        Ok(claims)
    }

    /// Validate refresh token specifically
    pub fn validate_refresh_token(&self, token: &str) -> Result<Claims> {
        let claims = self.validate_token(token)?;

        if claims.token_type != TokenType::Refresh {
            return Err(AuthError::InvalidToken(
                "Token is not a refresh token".to_string(),
            ));
        }

        Ok(claims)
    }

    /// Validate enrollment token specifically
    pub fn validate_enrollment_token(&self, token: &str) -> Result<Claims> {
        let claims = self.validate_token(token)?;

        if claims.token_type != TokenType::Enrollment {
            return Err(AuthError::InvalidToken(
                "Token is not an enrollment token".to_string(),
            ));
        }

        Ok(claims)
    }

    /// Extract claims without validation (for debugging, use with caution)
    pub fn decode_without_validation(&self, token: &str) -> Result<Claims> {
        let mut validation = Validation::new(self.algorithm);
        validation.insecure_disable_signature_validation();
        validation.validate_exp = false;

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;
        Ok(token_data.claims)
    }

    /// Get token expiration time
    pub fn get_expiration(&self, token: &str) -> Result<i64> {
        let claims = self.decode_without_validation(token)?;
        Ok(claims.exp)
    }
}

/// Generate a SHA256 hash of a token (for storing in database)
pub fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_validate_access_token() {
        let jwt = JwtService::new("test-secret-key-min-32-characters-long");
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        let email = "test@example.com";

        let token = jwt
            .generate_access_token_legacy(user_id, tenant_id, email)
            .expect("Failed to generate token");

        let claims = jwt
            .validate_access_token(&token)
            .expect("Failed to validate token");

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.tenant_id, Some(tenant_id.to_string()));
        assert_eq!(claims.email, email);
        assert_eq!(claims.token_type, TokenType::Access);
    }

    #[test]
    fn test_generate_and_validate_refresh_token() {
        let jwt = JwtService::new("test-secret-key-min-32-characters-long");
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        let email = "test@example.com";

        let token = jwt
            .generate_refresh_token_legacy(user_id, tenant_id, email)
            .expect("Failed to generate token");

        let claims = jwt
            .validate_refresh_token(&token)
            .expect("Failed to validate token");

        assert_eq!(claims.token_type, TokenType::Refresh);
    }

    #[test]
    fn test_invalid_token_type() {
        let jwt = JwtService::new("test-secret-key-min-32-characters-long");
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        let email = "test@example.com";

        let refresh_token = jwt
            .generate_refresh_token_legacy(user_id, tenant_id, email)
            .unwrap();

        // Try to validate refresh token as access token
        let result = jwt.validate_access_token(&refresh_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_hierarchical_platform_admin_token() {
        let jwt = JwtService::new("test-secret-key-min-32-characters-long");
        let user_id = Uuid::new_v4();
        let email = "admin@yoursaas.com";

        // Platform admin with no organization
        let token = jwt
            .generate_access_token(user_id, email, None, None, None, true)
            .expect("Failed to generate token");

        let claims = jwt
            .validate_access_token(&token)
            .expect("Failed to validate token");

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.organization_id, None);
        assert_eq!(claims.is_platform_admin, true);
        assert_eq!(claims.email, email);
    }

    #[test]
    fn test_hierarchical_org_member_token() {
        let jwt = JwtService::new("test-secret-key-min-32-characters-long");
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let email = "john@acme.com";

        // Org member with admin role
        let token = jwt
            .generate_access_token(
                user_id,
                email,
                Some(org_id),
                Some("acme-corp".to_string()),
                Some("admin".to_string()),
                false,
            )
            .expect("Failed to generate token");

        let claims = jwt
            .validate_access_token(&token)
            .expect("Failed to validate token");

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.organization_id, Some(org_id.to_string()));
        assert_eq!(claims.organization_slug, Some("acme-corp".to_string()));
        assert_eq!(claims.role, Some("admin".to_string()));
        assert_eq!(claims.is_platform_admin, false);
        assert_eq!(claims.email, email);
    }

    #[test]
    fn test_hash_token() {
        let token = "some-jwt-token";
        let hash1 = hash_token(token);
        let hash2 = hash_token(token);

        // Same token should produce same hash
        assert_eq!(hash1, hash2);

        // Different token should produce different hash
        let hash3 = hash_token("different-token");
        assert_ne!(hash1, hash3);
    }
}
