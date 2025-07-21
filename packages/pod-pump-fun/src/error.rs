//! Error types for the pump-fun pod

use thiserror::Error;

/// Main error type for the pump-fun pod
#[derive(Debug, Error)]
pub enum PumpFunError {
    /// Solana client errors
    #[error("Solana client error: {0}")]
    SolanaClient(String),
    
    /// Invalid bonding curve state
    #[error("Invalid bonding curve state: {0}")]
    InvalidBondingCurve(String),
    
    /// Insufficient liquidity
    #[error("Insufficient liquidity: need {need} SOL, have {have} SOL")]
    InsufficientLiquidity { need: f64, have: f64 },
    
    /// Slippage exceeded
    #[error("Slippage exceeded: expected {expected}, got {actual}")]
    SlippageExceeded { expected: f64, actual: f64 },
    
    /// Token already graduated
    #[error("Token already graduated to Raydium")]
    TokenGraduated,
    
    /// Risk limit exceeded
    #[error("Risk limit exceeded: {0}")]
    RiskLimitExceeded(String),
    
    /// Transaction failed
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Program error
    #[error("Program error: {0}")]
    ProgramError(String),
    
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

/// Convert String to PumpFunError
impl From<String> for PumpFunError {
    fn from(s: String) -> Self {
        PumpFunError::Internal(s)
    }
}

/// Convert &str to PumpFunError
impl From<&str> for PumpFunError {
    fn from(s: &str) -> Self {
        PumpFunError::Internal(s.to_string())
    }
}

/// Result type alias for pump-fun operations
pub type Result<T> = std::result::Result<T, PumpFunError>;

/// Convert PumpFunError to CoreError
impl From<PumpFunError> for banshee_core::error::CoreError {
    fn from(err: PumpFunError) -> Self {
        use banshee_core::error::CoreError;
        match err {
            PumpFunError::Core(core_err) => core_err,
            PumpFunError::Io(io_err) => CoreError::Io(io_err),
            PumpFunError::Json(json_err) => CoreError::Json(json_err),
            PumpFunError::ConfigError(msg) => CoreError::Config(msg),
            PumpFunError::Internal(msg) => CoreError::Internal(msg),
            other => CoreError::Plugin(format!("Pump.fun error: {}", other)),
        }
    }
}