//! # Banshee Runtime
//!
//! Modern runtime system with character sheet loading, embedded database support,
//! and comprehensive plugin architecture for AI agents.

#![allow(dead_code)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::new_without_default)]
#![allow(clippy::unwrap_or_default)]

pub mod character_sheet;
pub mod database;
pub mod embedded_db;
pub mod error;
pub mod redis;
pub mod retry;
pub mod ai_sdk_client;
pub mod key_manager;
pub mod http_pool;

// Re-export error types
pub use error::{RuntimeError, Result};

// Re-export retry functionality
pub use retry::{retry, retry_with_config, RetryConfig, RetryableError};

// Re-export main types for convenience
pub use character_sheet::{CharacterSheet, CharacterSheetManager};
pub use database::{DatabaseConfig, DatabaseManager, DatabaseStats};
pub use embedded_db::{
    DatabaseType, EmbeddedDatabase, EmbeddedDatabaseConfig, EmbeddedDatabaseManager, QueryResult,
};
pub use redis::{RedisConfig, RedisManager, RedisStats};
pub use ai_sdk_client::AiSdkClient;
pub use key_manager::{KeyManager, SecureKey};
