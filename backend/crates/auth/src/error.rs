use thiserror::Error;

pub type Result<T> = std::result::Result<T, AuthError>;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("Database error: {0}")]
    Database(String),

    #[error("User account is inactive")]
    UserInactive,

    #[error("Email not verified")]
    EmailNotVerified,

    #[error("MFA required")]
    MfaRequired { mfa_token: String },

    #[error("Invalid MFA code")]
    InvalidMfaCode,

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Password too weak: {0}")]
    WeakPassword(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] ciam_database::DatabaseError),

    #[error("Cache error: {0}")]
    CacheError(#[from] ciam_cache::CacheError),

    #[error("Password hashing error: {0}")]
    PasswordHashError(String),

    #[error("JWT error: {0}")]
    JwtError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("External provider error: {0}")]
    ExternalProviderError(String),

    #[error("Account locked until {locked_until}")]
    AccountLocked {
        locked_until: chrono::DateTime<chrono::Utc>,
    },

    #[error("Account banned: {0}")]
    AccountBanned(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

impl From<argon2::password_hash::Error> for AuthError {
    fn from(err: argon2::password_hash::Error) -> Self {
        AuthError::PasswordHashError(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind;
        match err.kind() {
            ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::JwtError(err.to_string()),
        }
    }
}

impl From<validator::ValidationErrors> for AuthError {
    fn from(err: validator::ValidationErrors) -> Self {
        AuthError::ValidationError(err.to_string())
    }
}

impl From<sqlx::Error> for AuthError {
    fn from(err: sqlx::Error) -> Self {
        AuthError::Internal(err.to_string())
    }
}
