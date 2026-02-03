use thiserror::Error;

pub type Result<T> = std::result::Result<T, AuthzError>;

#[derive(Debug, Error)]
pub enum AuthzError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid input: {0}")]
    ValidationError(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Cache error: {0}")]
    CacheError(String),
}

impl From<ciam_cache::CacheError> for AuthzError {
    fn from(err: ciam_cache::CacheError) -> Self {
        AuthzError::CacheError(err.to_string())
    }
}
