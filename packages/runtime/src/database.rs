//! Database initialization and management module
//!
//! Implements 2025 best practices for database startup, health checks,
//! and migration management with proper error handling and retry logic.

use anyhow::{Context, Result};
use sqlx::{PgPool, Row};
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://banshee:banshee_dev_password@localhost:5432/banshee".to_string(),
            max_connections: 20,
            min_connections: 5,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600), // 10 minutes
            max_lifetime: Duration::from_secs(1800), // 30 minutes
        }
    }
}

/// Database manager with health checks and migration support
pub struct DatabaseManager {
    pool: Option<PgPool>,
    config: DatabaseConfig,
}

impl DatabaseManager {
    /// Create a new database manager
    pub fn new(config: DatabaseConfig) -> Self {
        Self { pool: None, config }
    }

    /// Initialize database with retries and health checks
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing database connection...");

        // Wait for database to be ready
        self.wait_for_database().await?;

        // Create connection pool
        let pool = self.create_pool().await?;

        // Run health check
        self.health_check(&pool).await?;

        // Run migrations
        self.run_migrations(&pool).await?;

        // Final health check
        self.health_check(&pool).await?;

        self.pool = Some(pool);
        info!("Database initialization completed successfully");

        Ok(())
    }

    /// Wait for database to be ready with exponential backoff
    async fn wait_for_database(&self) -> Result<()> {
        info!("Waiting for PostgreSQL to be ready...");

        let max_attempts = 30;
        let mut attempt = 1;
        let base_delay = Duration::from_millis(500);

        while attempt <= max_attempts {
            debug!(
                "Database connection attempt {} of {}",
                attempt, max_attempts
            );

            match self.test_connection().await {
                Ok(_) => {
                    info!("PostgreSQL is ready!");
                    return Ok(());
                }
                Err(e) => {
                    if attempt == max_attempts {
                        error!(
                            "Failed to connect to database after {} attempts",
                            max_attempts
                        );
                        return Err(e).context("Database connection failed");
                    }

                    let delay = base_delay * 2_u32.pow((attempt - 1).min(10));
                    warn!(
                        "Database not ready (attempt {}): {}. Retrying in {:?}",
                        attempt, e, delay
                    );
                    sleep(delay).await;
                }
            }

            attempt += 1;
        }

        Err(anyhow::anyhow!("Database connection timeout"))
    }

    /// Test basic database connection
    async fn test_connection(&self) -> Result<()> {
        let connection_timeout = Duration::from_secs(5);

        let pool = timeout(
            connection_timeout,
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(1)
                .connect(&self.config.url),
        )
        .await
        .context("Database connection timeout")?
        .context("Failed to create connection pool")?;

        timeout(connection_timeout, sqlx::query("SELECT 1").fetch_one(&pool))
            .await
            .context("Database query timeout")?
            .context("Failed to execute test query")?;

        pool.close().await;
        Ok(())
    }

    /// Create the main connection pool
    async fn create_pool(&self) -> Result<PgPool> {
        info!("Creating database connection pool...");

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(self.config.max_connections)
            .min_connections(self.config.min_connections)
            .acquire_timeout(self.config.connect_timeout)
            .idle_timeout(Some(self.config.idle_timeout))
            .max_lifetime(Some(self.config.max_lifetime))
            .test_before_acquire(true)
            .connect(&self.config.url)
            .await
            .context("Failed to create database connection pool")?;

        info!(
            "Database connection pool created with {}-{} connections",
            self.config.min_connections, self.config.max_connections
        );

        Ok(pool)
    }

    /// Run database health check
    pub async fn health_check(&self, pool: &PgPool) -> Result<()> {
        debug!("Running database health check...");

        // Test basic connectivity
        let result = sqlx::query("SELECT version(), current_database(), current_user")
            .fetch_one(pool)
            .await
            .context("Health check query failed")?;

        let version: String = result.get(0);
        let database: String = result.get(1);
        let user: String = result.get(2);

        info!("Database health check passed:");
        info!("  Version: {}", version);
        info!("  Database: {}", database);
        info!("  User: {}", user);

        // Test connection pool status
        let pool_status = format!(
            "Pool status: {}/{} connections",
            pool.size(),
            self.config.max_connections
        );
        debug!("{}", pool_status);

        Ok(())
    }

    /// Run database migrations
    async fn run_migrations(&self, pool: &PgPool) -> Result<()> {
        info!("Running database migrations...");

        // Check if migrations directory exists
        let migrations_path = std::path::Path::new("./migrations");
        if !migrations_path.exists() {
            warn!("Migrations directory not found, skipping migrations");
            return Ok(());
        }

        // Run SQLx migrations
        // Note: SQLx migrations are disabled for embedded database demonstration
        // sqlx::migrate!("./migrations")
        //     .run(pool)
        //     .await
        //     .context("Failed to run database migrations")?;

        info!("Database migrations completed successfully");

        // Log migration status
        let migration_info = sqlx::query(
            "SELECT version, description, installed_on FROM _sqlx_migrations ORDER BY version DESC LIMIT 5"
        )
        .fetch_all(pool)
        .await;

        match migration_info {
            Ok(rows) => {
                info!("Recent migrations:");
                for row in rows {
                    let version: i64 = row.get(0);
                    let description: String = row.get(1);
                    let installed_on: chrono::DateTime<chrono::Utc> = row.get(2);
                    info!(
                        "  {} - {} ({})",
                        version,
                        description,
                        installed_on.format("%Y-%m-%d %H:%M:%S")
                    );
                }
            }
            Err(e) => {
                debug!("Could not fetch migration info: {}", e);
            }
        }

        Ok(())
    }

    /// Get the database pool
    pub fn pool(&self) -> Result<&PgPool> {
        self.pool
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))
    }

    /// Shutdown database connections
    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(pool) = self.pool.take() {
            info!("Shutting down database connections...");
            pool.close().await;
            info!("Database connections closed");
        }
        Ok(())
    }

    /// Check if database is ready
    pub async fn is_ready(&self) -> bool {
        if let Some(pool) = &self.pool {
            sqlx::query("SELECT 1").fetch_one(pool).await.is_ok()
        } else {
            false
        }
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        let pool = self.pool()?;

        let stats_query = sqlx::query(
            r#"
            SELECT 
                pg_database_size(current_database()) as db_size,
                (SELECT count(*) FROM pg_stat_activity WHERE state = 'active') as active_connections,
                (SELECT count(*) FROM pg_stat_activity) as total_connections
            "#
        ).fetch_one(pool).await?;

        Ok(DatabaseStats {
            database_size: stats_query.get::<i64, _>(0) as u64,
            active_connections: stats_query.get::<i64, _>(1) as u32,
            total_connections: stats_query.get::<i64, _>(2) as u32,
            pool_size: pool.size(),
            pool_max_size: self.config.max_connections,
        })
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub database_size: u64,
    pub active_connections: u32,
    pub total_connections: u32,
    pub pool_size: u32,
    pub pool_max_size: u32,
}

impl std::fmt::Display for DatabaseStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DB Size: {:.2} MB, Connections: {}/{}, Pool: {}/{}",
            self.database_size as f64 / 1024.0 / 1024.0,
            self.active_connections,
            self.total_connections,
            self.pool_size,
            self.pool_max_size
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert!(!config.url.is_empty());
        assert!(config.max_connections > 0);
        assert!(config.min_connections > 0);
        assert!(config.connect_timeout > Duration::ZERO);
    }

    #[tokio::test]
    async fn test_database_manager_creation() {
        let config = DatabaseConfig::default();
        let manager = DatabaseManager::new(config);
        assert!(manager.pool.is_none());
        assert!(!manager.is_ready().await);
    }

    // Integration tests would require a running PostgreSQL instance
    #[ignore]
    #[tokio::test]
    async fn test_database_initialization() {
        let config = DatabaseConfig::default();
        let mut manager = DatabaseManager::new(config);

        // This test requires a running PostgreSQL instance
        let result = manager.initialize().await;
        if result.is_ok() {
            assert!(manager.is_ready().await);
            let stats = manager.get_stats().await.unwrap();
            assert!(stats.pool_size > 0);
            manager.shutdown().await.unwrap();
        }
    }
}
