//! Error types for the jito-mev pod

use thiserror::Error;

/// Main error type for the jito-mev pod
#[derive(Debug, Error)]
pub enum JitoError {
    /// Bundle submission failed
    #[error("Bundle submission failed: {0}")]
    BundleSubmissionFailed(String),
    
    /// Tip amount too low
    #[error("Tip amount too low: minimum {min} SOL, provided {provided} SOL")]
    TipTooLow { min: f64, provided: f64 },
    
    /// MEV opportunity expired
    #[error("MEV opportunity expired")]
    OpportunityExpired,
    
    /// Insufficient profit margin
    #[error("Insufficient profit margin: expected {expected}%, got {actual}%")]
    InsufficientProfit { expected: f64, actual: f64 },
    
    /// Bundle simulation failed
    #[error("Bundle simulation failed: {0}")]
    SimulationFailed(String),
    
    /// TipRouter error
    #[error("TipRouter error: {0}")]
    TipRouterError(String),
    
    /// Risk limit exceeded
    #[error("Risk limit exceeded: {0}")]
    RiskLimitExceeded(String),
    
    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// FFI error
    #[error("FFI error: {0}")]
    FfiError(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// Core framework error
    #[error("Core error: {0}")]
    Core(#[from] banshee_core::error::CoreError),
    
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Convert String to JitoError
impl From<String> for JitoError {
    fn from(s: String) -> Self {
        JitoError::Internal(s)
    }
}

/// Convert &str to JitoError
impl From<&str> for JitoError {
    fn from(s: &str) -> Self {
        JitoError::Internal(s.to_string())
    }
}

/// Result type alias for jito operations
pub type Result<T> = std::result::Result<T, JitoError>;

/// Convert JitoError to CoreError
impl From<JitoError> for banshee_core::error::CoreError {
    fn from(err: JitoError) -> Self {
        use banshee_core::error::CoreError;
        match err {
            JitoError::Core(core_err) => core_err,
            JitoError::Io(io_err) => CoreError::Io(io_err),
            JitoError::Json(json_err) => CoreError::Json(json_err),
            JitoError::ConfigError(msg) => CoreError::Config(msg),
            JitoError::Internal(msg) => CoreError::Internal(msg),
            other => CoreError::Plugin(format!("Jito MEV error: {}", other)),
        }
    }
}