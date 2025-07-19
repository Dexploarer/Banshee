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
pub mod redis;

/// Result type used throughout the runtime framework
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// Re-export main types for convenience
pub use character_sheet::{CharacterSheet, CharacterSheetManager};
pub use database::{DatabaseConfig, DatabaseManager, DatabaseStats};
pub use embedded_db::{
    DatabaseType, EmbeddedDatabase, EmbeddedDatabaseConfig, EmbeddedDatabaseManager, QueryResult,
};
pub use redis::{RedisConfig, RedisManager, RedisStats};
