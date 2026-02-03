use thiserror::Error;

pub type Result<T> = std::result::Result<T, CacheError>;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Redis connection error: {0}")]
    ConnectionError(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Cache key not found: {0}")]
    NotFound(String),

    #[error("Cache error: {0}")]
    Other(String),
}
