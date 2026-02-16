use thiserror::Error;

pub type Result<T> = std::result::Result<T, DatabaseError>;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    ConnectionError(#[from] sqlx::Error),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Database error: {0}")]
    Other(String),
}

impl DatabaseError {
    pub fn not_found(entity: &str, id: &str) -> Self {
        Self::NotFound(format!("{} with id {} not found", entity, id))
    }

    pub fn duplicate(entity: &str, field: &str) -> Self {
        Self::DuplicateEntry(format!("{} with {} already exists", entity, field))
    }
}

// Note: From<sqlx::Error> is automatically implemented via #[from] attribute above
