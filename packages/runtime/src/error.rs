//! Error types for the runtime module

use thiserror::Error;

/// Main error type for the runtime module
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// Character sheet related errors
    #[error("Character sheet error: {0}")]
    CharacterSheet(String),
    
    /// Database operation errors
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    /// Redis operation errors
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    /// Key management errors
    #[error("Key management error: {0}")]
    KeyManagement(String),
    
    /// Transport errors
    #[error("Transport error: {0}")]
    Transport(String),
    
    /// AI SDK client errors
    #[error("AI SDK error: {0}")]
    AiSdk(String),
    
    /// Plugin loading errors
    #[error("Plugin error: {0}")]
    Plugin(String),
    
    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),
    
    /// Generic internal errors
    #[error("Internal error: {0}")]
    Internal(String),
    
    /// Network errors
    #[error("Network error: {0}")]
    Network(String),
    
    /// Timeout errors
    #[error("Operation timed out: {0}")]
    Timeout(String),
    
    /// Authentication errors
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    /// Not found errors
    #[error("{0} not found")]
    NotFound(String),
    
    /// Already exists errors
    #[error("{0} already exists")]
    AlreadyExists(String),
    
    /// Invalid state errors
    #[error("Invalid state: {0}")]
    InvalidState(String),
}

/// Convert string errors to RuntimeError
impl From<String> for RuntimeError {
    fn from(s: String) -> Self {
        RuntimeError::Internal(s)
    }
}

/// Convert &str errors to RuntimeError
impl From<&str> for RuntimeError {
    fn from(s: &str) -> Self {
        RuntimeError::Internal(s.to_string())
    }
}

/// Result type alias for runtime operations
pub type Result<T> = std::result::Result<T, RuntimeError>;