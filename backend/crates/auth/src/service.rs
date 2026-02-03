use crate::account_lockout::AccountLockoutService;
use crate::error::{AuthError, Result};
use crate::jwt::{hash_token, Claims, JwtService};
use crate::password::PasswordHasher;
use chrono::Utc;
use ciam_cache::Cache;
use ciam_database::{Database, RoleRepository, SessionRepository, UserRepository};
use ciam_models::user::{NewUser, UserProfile};
use ciam_models::{NewSession, User};
use serde::{Deserialize, Serialize};
use tracing;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    pub tenant_id: Uuid,

    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: String,

    pub phone: Option<String>,
    pub metadata: Option<ciam_models::UserMetadata>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    pub tenant_id: Uuid,

    #[validate(email)]
    pub email: String,

    pub password: String,

    // Optional context
    pub device_fingerprint: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Hierarchical login request with optional organization context
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct HierarchicalLoginRequest {
    #[validate(email)]
    pub email: String,

    pub password: String,

    /// Optional organization slug (e.g., "acme-corp")
    /// If provided, user must be a member of this organization
    pub organization_slug: Option<String>,

    /// Optional organization ID (alternative to slug)
    pub organization_id: Option<Uuid>,

    // Optional context
    pub device_fingerprint: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum AuthResponse {
    #[serde(rename = "success")]
    Success {
        access_token: String,
        refresh_token: String,
        token_type: String,
        expires_in: i64,
        user: UserProfile,
    },
    #[serde(rename = "mfa_required")]
    MfaRequired {
        challenge_token: String,
        methods: Vec<String>,
        message: String,
    },
    #[serde(rename = "mfa_enrollment_required")]
    MfaEnrollmentRequired {
        enrollment_token: String,
        message: String,
        grace_period_expires: Option<chrono::DateTime<Utc>>,
        can_skip: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

pub struct AuthService {
    pub db: Database,
    pub cache: Cache,
    pub jwt: JwtService,
    user_repo: UserRepository,
    session_repo: SessionRepository,
    role_repo: RoleRepository,
    lockout: AccountLockoutService,
    org_member_repo: ciam_database::OrganizationMemberRepository,
    tenant_repo: ciam_database::TenantRepository,
}

impl AuthService {
    pub fn new(db: Database, cache: Cache, jwt: JwtService) -> Self {
        let pool = db.pool().clone();

        Self {
            lockout: AccountLockoutService::new(db.clone()),
            db,
            cache,
            jwt,
            user_repo: UserRepository::new(pool.clone()),
            session_repo: SessionRepository::new(pool.clone()),
            role_repo: RoleRepository::new(pool.clone()),
            org_member_repo: ciam_database::OrganizationMemberRepository::new(pool.clone()),
            tenant_repo: ciam_database::TenantRepository::new(pool),
        }
    }

    /// Register a new user
    pub async fn register(&self, request: RegisterRequest) -> Result<AuthResponse> {
        // Validate input
        request.validate()?;

        // Hash password
        let password_hash = PasswordHasher::hash(&request.password)?;

        // Create user in database
        let new_user = NewUser {
            tenant_id: Some(request.tenant_id),
            email: request.email.clone(),
            password: Some(request.password),
            phone: request.phone,
            metadata: request.metadata,
            is_platform_admin: false, // Regular user registration
        };

        let user = self.user_repo.create(&new_user, &password_hash).await?;

        // Generate tokens (using legacy method for backward compatibility)
        let access_token = self
            .jwt
            .generate_access_token_legacy(user.id, request.tenant_id, &user.email)?;

        let refresh_token = self
            .jwt
            .generate_refresh_token_legacy(user.id, request.tenant_id, &user.email)?;

        // Create session
        let _session = self
            .create_session(&user, &access_token, &refresh_token, None, None, None)
            .await?;

        // Cache user data
        self.cache_user(&user).await?;

        Ok(AuthResponse::Success {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600, // 1 hour in seconds
            user: user.into(),
        })
    }

    /// Login with email and password
    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse> {
        // Validate input
        request.validate()?;

        let ip_address = request.ip_address.as_deref().unwrap_or("unknown");
        let user_agent = request.user_agent.as_deref();

        // Check if IP or email is banned
        let is_banned = self
            .lockout
            .is_banned(Some(request.tenant_id), None, Some(&request.email), Some(ip_address))
            .await?;

        if is_banned {
            return Err(AuthError::AccountBanned(
                "This account or IP address has been banned".to_string(),
            ));
        }

        // Find user
        let user = self
            .user_repo
            .find_by_email(request.tenant_id, &request.email)
            .await
            .map_err(|_| AuthError::InvalidCredentials)?;

        // Check if account is locked
        if let Some(locked_until) = self.lockout.is_locked(user.id).await? {
            return Err(AuthError::AccountLocked { locked_until });
        }

        // Check if user is active
        if !user.is_active {
            return Err(AuthError::UserInactive);
        }

        // Get tenant security settings for lockout policy
        let tenant_settings: Option<sqlx::types::Json<ciam_models::TenantSettings>> =
            sqlx::query_scalar("SELECT settings FROM organizations WHERE id = $1")
                .bind(request.tenant_id)
                .fetch_optional(self.db.pool())
                .await?;

        let security = tenant_settings
            .as_ref()
            .map(|s| s.0.security.clone())
            .unwrap_or_default();

        // Verify password
        let password_hash = user
            .password_hash
            .as_ref()
            .ok_or(AuthError::InvalidCredentials)?;

        let is_valid = PasswordHasher::verify(&request.password, password_hash)?;

        if !is_valid {
            // Handle failed login attempt (may lock account)
            self.lockout
                .handle_failed_login(
                    user.id,
                    Some(request.tenant_id),
                    &request.email,
                    ip_address,
                    user_agent,
                    security.max_login_attempts,
                    security.lockout_duration_minutes,
                )
                .await?;

            return Err(AuthError::InvalidCredentials);
        }

        // Record successful login
        self.lockout
            .record_attempt(
                Some(user.id),
                Some(request.tenant_id),
                &request.email,
                ip_address,
                user_agent,
                true,
                None,
            )
            .await?;

        // Check tenant MFA policy
        let tenant_settings: Option<sqlx::types::Json<ciam_models::TenantSettings>> =
            sqlx::query_scalar("SELECT settings FROM tenants WHERE id = $1")
                .bind(request.tenant_id)
                .fetch_optional(self.db.pool())
                .await?;

        let mfa_required = tenant_settings
            .as_ref()
            .map(|s| s.0.security.mfa_required)
            .unwrap_or(false);

        // Check if user has MFA enabled
        if mfa_required && !user.mfa_enabled {
            // Generate enrollment token for MFA setup
            let enrollment_token = self.jwt.generate_enrollment_token(
                user.id,
                &user.email,
                Some(request.tenant_id),
            )?;

            // Check if grace period has expired
            if let Some(enforced_at) = user.mfa_enforced_at {
                if Utc::now() > enforced_at {
                    // Grace period expired - block login
                    return Ok(AuthResponse::MfaEnrollmentRequired {
                        enrollment_token,
                        message: "Multi-factor authentication is required for your account. Please enroll before you can continue.".to_string(),
                        grace_period_expires: Some(enforced_at),
                        can_skip: false,
                    });
                } else {
                    // Within grace period - warn but allow login
                    tracing::warn!(
                        "User {} is within MFA grace period, expires at {}",
                        user.id,
                        enforced_at
                    );
                }
            } else {
                // First login after MFA enforcement - set grace period
                let grace_days = tenant_settings
                    .as_ref()
                    .map(|s| s.0.security.mfa_grace_period_days)
                    .unwrap_or(7);

                let grace_period_expires = Utc::now() + chrono::Duration::days(grace_days as i64);

                sqlx::query("UPDATE users SET mfa_enforced_at = $1 WHERE id = $2")
                    .bind(grace_period_expires)
                    .bind(user.id)
                    .execute(self.db.pool())
                    .await?;

                return Ok(AuthResponse::MfaEnrollmentRequired {
                    enrollment_token,
                    message: format!("Multi-factor authentication will be required in {} days. Please enroll soon.", grace_days),
                    grace_period_expires: Some(grace_period_expires),
                    can_skip: true,
                });
            }
        }

        // If user has MFA enabled, require verification
        if user.mfa_enabled {
            // Create MFA challenge
            let challenge_token = uuid::Uuid::new_v4().to_string();

            // Get user's verified MFA methods
            let methods: Vec<String> = sqlx::query_scalar(
                "SELECT method_type FROM mfa_methods WHERE user_id = $1 AND verified = true",
            )
            .bind(user.id)
            .fetch_all(self.db.pool())
            .await?;

            if methods.is_empty() {
                return Err(AuthError::Internal(
                    "MFA is enabled but no verified methods found".to_string(),
                ));
            }

            // Store challenge
            let expires_at = Utc::now() + chrono::Duration::minutes(5);
            sqlx::query(
                r#"
                INSERT INTO mfa_challenges (user_id, challenge_token, expires_at, ip_address, user_agent)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(user.id)
            .bind(&challenge_token)
            .bind(expires_at)
            .bind(request.ip_address.as_deref())
            .bind(request.user_agent.as_deref())
            .execute(self.db.pool())
            .await?;

            return Ok(AuthResponse::MfaRequired {
                challenge_token,
                methods,
                message: "Please verify your identity with a second factor.".to_string(),
            });
        }

        // Update last login
        self.user_repo.update_last_login(user.id).await?;

        // Generate tokens (using legacy method for backward compatibility)
        let access_token = self
            .jwt
            .generate_access_token_legacy(user.id, request.tenant_id, &user.email)?;

        let refresh_token = self
            .jwt
            .generate_refresh_token_legacy(user.id, request.tenant_id, &user.email)?;

        // Create session
        self.create_session(
            &user,
            &access_token,
            &refresh_token,
            request.device_fingerprint,
            request.ip_address,
            request.user_agent,
        )
        .await?;

        // Cache user data
        self.cache_user(&user).await?;

        Ok(AuthResponse::Success {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            user: user.into(),
        })
    }

    /// Hierarchical login with optional organization context
    /// Supports platform admins (no org) and org members
    pub async fn login_hierarchical(&self, request: HierarchicalLoginRequest) -> Result<AuthResponse> {
        // Validate input
        request.validate()?;

        let ip_address = request.ip_address.as_deref().unwrap_or("unknown");
        let user_agent = request.user_agent.as_deref();

        // Resolve organization ID from slug if provided
        let organization_id = if let Some(slug) = &request.organization_slug {
            let org = self
                .tenant_repo
                .find_by_slug(slug)
                .await
                .map_err(|_| AuthError::NotFound(format!("Organization '{}' not found", slug)))?;
            Some(org.id)
        } else {
            request.organization_id
        };

        // Find user by email (global lookup, not tenant-scoped)
        let user = sqlx::query_as::<_, ciam_models::User>(
            "SELECT * FROM users WHERE email = $1"
        )
        .bind(&request.email)
        .fetch_optional(self.db.pool())
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

        // Check if IP or email is banned (use organization_id if provided, else None)
        let is_banned = self
            .lockout
            .is_banned(
                organization_id,  // Pass None for platform admins
                None,
                Some(&request.email),
                Some(ip_address),
            )
            .await?;

        if is_banned {
            return Err(AuthError::AccountBanned(
                "This account or IP address has been banned".to_string(),
            ));
        }

        // Check if account is locked
        if let Some(locked_until) = self.lockout.is_locked(user.id).await? {
            return Err(AuthError::AccountLocked { locked_until });
        }

        // Check if user is active
        if !user.is_active {
            return Err(AuthError::UserInactive);
        }

        // Verify password
        let password_hash = user
            .password_hash
            .as_ref()
            .ok_or(AuthError::InvalidCredentials)?;

        let is_valid = crate::password::PasswordHasher::verify(&request.password, password_hash)?;

        if !is_valid {
            // Handle failed login attempt
            self.lockout
                .handle_failed_login(
                    user.id,
                    organization_id,  // Pass None for platform admins
                    &request.email,
                    ip_address,
                    user_agent,
                    5, // default max attempts
                    30, // default lockout duration
                )
                .await?;

            return Err(AuthError::InvalidCredentials);
        }

        // Record successful login
        self.lockout
            .record_attempt(
                Some(user.id),
                organization_id,  // Pass None for platform admins
                &request.email,
                ip_address,
                user_agent,
                true,
                None,
            )
            .await?;

        // Determine organization context and role
        let (org_id, org_slug, role) = if let Some(org_id) = organization_id {
            // Verify user is a member of the organization
            let membership = self
                .org_member_repo
                .get_member(user.id, org_id)
                .await?
                .ok_or_else(|| {
                    AuthError::Forbidden(format!(
                        "User is not a member of organization '{}'",
                        request.organization_slug.as_deref().unwrap_or("unknown")
                    ))
                })?;

            // Get organization details for slug
            let org = self.tenant_repo.find_by_id(org_id).await?;

            (Some(org_id), Some(org.slug), Some(membership.role))
        } else {
            // No organization context - must be platform admin
            if !user.is_platform_admin {
                return Err(AuthError::Forbidden(
                    "Organization context is required for non-admin users".to_string(),
                ));
            }
            (None, None, None)
        };

        // Check organization MFA enforcement policy
        if let Some(org_id) = org_id {
            // Fetch organization security settings
            let org_settings: Option<(serde_json::Value,)> = sqlx::query_as(
                "SELECT settings FROM organizations WHERE id = $1"
            )
            .bind(org_id)
            .fetch_optional(self.db.pool())
            .await?;

            if let Some((settings_json,)) = org_settings {
                // Parse settings
                if let Ok(settings) = serde_json::from_value::<ciam_models::OrganizationSettings>(settings_json) {
                    // Check if MFA is required for this organization
                    if settings.security.mfa_required {
                        // Check if user has any verified MFA methods
                        let has_mfa: bool = sqlx::query_scalar(
                            "SELECT EXISTS(SELECT 1 FROM mfa_methods WHERE user_id = $1 AND verified = true)"
                        )
                        .bind(user.id)
                        .fetch_one(self.db.pool())
                        .await?;

                        if !has_mfa {
                            // User doesn't have MFA set up
                            // Calculate grace period expiration
                            let grace_period_expires = if let Some(enforcement_date) = settings.security.mfa_enforcement_date {
                                let grace_days = settings.security.mfa_grace_period_days;
                                Some(enforcement_date + chrono::Duration::days(grace_days as i64))
                            } else {
                                None
                            };

                            // Check if grace period has expired
                            let grace_expired = if let Some(expires) = grace_period_expires {
                                chrono::Utc::now() > expires
                            } else {
                                true // No enforcement date means enforce immediately
                            };

                            // Generate enrollment token for MFA setup
                            let enrollment_token = self.jwt.generate_enrollment_token(
                                user.id,
                                &user.email,
                                Some(org_id),
                            )?;

                            if grace_expired {
                                // Grace period expired - must set up MFA to continue
                                return Ok(AuthResponse::MfaEnrollmentRequired {
                                    enrollment_token,
                                    message: "Your organization requires multi-factor authentication. Please set up MFA to continue.".to_string(),
                                    grace_period_expires: None,
                                    can_skip: false,
                                });
                            } else {
                                // Still in grace period - allow login but warn
                                return Ok(AuthResponse::MfaEnrollmentRequired {
                                    enrollment_token,
                                    message: "Your organization requires multi-factor authentication. Please set up MFA soon.".to_string(),
                                    grace_period_expires,
                                    can_skip: true,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Check MFA requirements (simplified for now - would check tenant settings if org provided)
        if user.mfa_enabled {
            // Create MFA challenge
            let challenge_token = uuid::Uuid::new_v4().to_string();

            // Get user's verified MFA methods
            let methods: Vec<String> = sqlx::query_scalar(
                "SELECT method_type FROM mfa_methods WHERE user_id = $1 AND verified = true",
            )
            .bind(user.id)
            .fetch_all(self.db.pool())
            .await?;

            if methods.is_empty() {
                return Err(AuthError::Internal(
                    "MFA is enabled but no verified methods found".to_string(),
                ));
            }

            // Store challenge
            let expires_at = chrono::Utc::now() + chrono::Duration::minutes(5);
            sqlx::query(
                r#"
                INSERT INTO mfa_challenges (user_id, challenge_token, expires_at, ip_address, user_agent)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(user.id)
            .bind(&challenge_token)
            .bind(expires_at)
            .bind(request.ip_address.as_deref())
            .bind(request.user_agent.as_deref())
            .execute(self.db.pool())
            .await?;

            return Ok(AuthResponse::MfaRequired {
                challenge_token,
                methods,
                message: "Please verify your identity with a second factor.".to_string(),
            });
        }

        // Update last login
        self.user_repo.update_last_login(user.id).await?;

        // Generate tokens with hierarchical context
        let access_token = self.jwt.generate_access_token(
            user.id,
            &user.email,
            org_id,
            org_slug.clone(),
            role.clone(),
            user.is_platform_admin,
        )?;

        let refresh_token = self.jwt.generate_refresh_token(
            user.id,
            &user.email,
            org_id,
            org_slug,
            role,
            user.is_platform_admin,
        )?;

        // Create session
        self.create_session(
            &user,
            &access_token,
            &refresh_token,
            request.device_fingerprint,
            request.ip_address,
            request.user_agent,
        )
        .await?;

        // Cache user data
        self.cache_user(&user).await?;

        Ok(AuthResponse::Success {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            user: user.into(),
        })
    }

    /// Refresh access token
    pub async fn refresh_token(&self, request: RefreshTokenRequest) -> Result<AuthResponse> {
        // Validate refresh token
        let claims = self.jwt.validate_refresh_token(&request.refresh_token)?;

        // Parse UUIDs
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AuthError::InvalidToken("Invalid user ID".to_string()))?;

        // Parse organization_id from claims if present
        let organization_id = claims.organization_id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok());

        // Check if session exists
        let token_hash = hash_token(&request.refresh_token);
        let _session = self
            .session_repo
            .find_by_refresh_token(&token_hash)
            .await?;

        // Get user
        let user = self.user_repo.find_by_id(user_id).await?;

        // Check if user is active
        if !user.is_active {
            return Err(AuthError::UserInactive);
        }

        // Generate new tokens preserving hierarchical context from original token
        let new_access_token = self.jwt.generate_access_token(
            user.id,
            &user.email,
            organization_id,
            claims.organization_slug.clone(),
            claims.role.clone(),
            claims.is_platform_admin,
        )?;

        let new_refresh_token = self.jwt.generate_refresh_token(
            user.id,
            &user.email,
            organization_id,
            claims.organization_slug,
            claims.role,
            claims.is_platform_admin,
        )?;

        // Delete old session
        self.session_repo.delete_by_token(&token_hash).await?;

        // Create new session
        self.create_session(&user, &new_access_token, &new_refresh_token, None, None, None)
            .await?;

        // Cache user data
        self.cache_user(&user).await?;

        Ok(AuthResponse::Success {
            access_token: new_access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            user: user.into(),
        })
    }

    /// Logout (invalidate session)
    pub async fn logout(&self, access_token: &str) -> Result<()> {
        let token_hash = hash_token(access_token);

        // Delete session
        self.session_repo.delete_by_token(&token_hash).await?;

        // Invalidate cache
        let claims = self.jwt.decode_without_validation(access_token)?;
        self.invalidate_user_cache(&claims.sub).await?;

        Ok(())
    }

    /// Validate access token and return claims
    pub async fn validate(&self, access_token: &str) -> Result<Claims> {
        // Validate JWT
        let claims = self.jwt.validate_access_token(access_token)?;

        // Check if session exists
        let token_hash = hash_token(access_token);
        self.session_repo.find_by_token(&token_hash).await?;

        Ok(claims)
    }

    /// Get user by ID with caching
    pub async fn get_user(&self, user_id: Uuid) -> Result<User> {
        // Try cache first
        let cache_key = ciam_cache::user_cache_key(&user_id.to_string());

        if let Some(user) = self.cache.get::<User>(&cache_key).await? {
            return Ok(user);
        }

        // Fetch from database
        let user = self.user_repo.find_by_id(user_id).await?;

        // Cache for 15 minutes
        self.cache.set(&cache_key, &user, Some(900)).await?;

        Ok(user)
    }

    /// Check if user has permission
    pub async fn check_permission(&self, user_id: Uuid, permission_name: &str) -> Result<bool> {
        // Try cache first
        let cache_key = format!("authz:{}:{}", user_id, permission_name);

        if let Some(has_permission) = self.cache.get::<bool>(&cache_key).await? {
            return Ok(has_permission);
        }

        // Check from database
        let has_permission = self
            .role_repo
            .user_has_permission(user_id, permission_name)
            .await?;

        // Cache for 5 seconds (short TTL for permissions)
        self.cache.set(&cache_key, &has_permission, Some(5)).await?;

        Ok(has_permission)
    }

    // Private helper methods

    async fn create_session(
        &self,
        user: &User,
        access_token: &str,
        refresh_token: &str,
        device_fingerprint: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<ciam_models::Session> {
        let access_token_hash = hash_token(access_token);
        let refresh_token_hash = hash_token(refresh_token);

        let exp_timestamp = self.jwt.get_expiration(refresh_token)?;
        let expires_at = chrono::DateTime::from_timestamp(exp_timestamp, 0)
            .ok_or_else(|| AuthError::Internal("Invalid expiration timestamp".to_string()))?;

        let new_session = NewSession {
            user_id: user.id,
            token_hash: access_token_hash,
            refresh_token_hash: Some(refresh_token_hash),
            device_fingerprint,
            ip_address,
            user_agent,
            expires_at: expires_at.into(),
        };

        let session = self.session_repo.create(&new_session).await?;

        Ok(session)
    }

    async fn cache_user(&self, user: &User) -> Result<()> {
        let cache_key = ciam_cache::user_cache_key(&user.id.to_string());
        self.cache.set(&cache_key, user, Some(900)).await?; // 15 minutes
        Ok(())
    }

    async fn invalidate_user_cache(&self, user_id: &str) -> Result<()> {
        let cache_key = ciam_cache::user_cache_key(user_id);
        self.cache.delete(&cache_key).await?;
        Ok(())
    }
}
