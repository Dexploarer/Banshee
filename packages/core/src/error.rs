//! Error types for the core module

use thiserror::Error;

/// Main error type for the core module
#[derive(Debug, Error)]
pub enum CoreError {
    /// Plugin-related errors
    #[error("Plugin error: {0}")]
    Plugin(String),
    
    /// Action execution errors
    #[error("Action error: {0}")]
    Action(String),
    
    /// Provider errors
    #[error("Provider error: {0}")]
    Provider(String),
    
    /// Evaluator errors
    #[error("Evaluator error: {0}")]
    Evaluator(String),
    
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),
    
    /// Version compatibility errors
    #[error("Version incompatibility: {0}")]
    VersionIncompatible(String),
    
    /// Dependency errors
    #[error("Dependency error: {0}")]
    Dependency(String),
    
    /// Not found errors
    #[error("{0} not found")]
    NotFound(String),
    
    /// Already exists errors
    #[error("{0} already exists")]
    AlreadyExists(String),
    
    /// Invalid state errors
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    /// Timeout errors
    #[error("Operation timed out: {0}")]
    Timeout(String),
    
    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    /// Generic internal errors
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Convert string errors to CoreError
impl From<String> for CoreError {
    fn from(s: String) -> Self {
        CoreError::Internal(s)
    }
}

/// Convert &str errors to CoreError
impl From<&str> for CoreError {
    fn from(s: &str) -> Self {
        CoreError::Internal(s.to_string())
    }
}

/// Result type alias for core operations
pub type Result<T> = std::result::Result<T, CoreError>;