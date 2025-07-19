use thiserror::Error;

/// Memory plugin error types
#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("UUID parse error: {0}")]
    UuidParse(#[from] uuid::Error),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for memory operations
pub type MemoryResult<T> = Result<T, MemoryError>;
