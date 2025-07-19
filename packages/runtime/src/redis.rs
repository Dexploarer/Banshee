//! Redis initialization and management module
//!
//! Implements connection management, health checks, and configuration
//! for Redis cache with proper error handling and monitoring.

use anyhow::{Context, Result};
use redis::aio::MultiplexedConnection;
use redis::{Client, RedisResult};
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};

/// Redis configuration
#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
    pub connect_timeout: Duration,
    pub response_timeout: Duration,
    pub reconnect_attempts: u32,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            max_connections: 10,
            connect_timeout: Duration::from_secs(10),
            response_timeout: Duration::from_secs(5),
            reconnect_attempts: 5,
        }
    }
}

/// Redis manager with health checks and connection management
pub struct RedisManager {
    client: Option<Client>,
    connection: Option<MultiplexedConnection>,
    config: RedisConfig,
}

impl RedisManager {
    /// Create a new Redis manager
    pub fn new(config: RedisConfig) -> Self {
        Self {
            client: None,
            connection: None,
            config,
        }
    }

    /// Initialize Redis connection with retries and health checks
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing Redis connection...");

        // Wait for Redis to be ready
        self.wait_for_redis().await?;

        // Create client and connection
        let client = self.create_client()?;
        let connection = self.create_connection(&client).await?;

        // Run health check
        self.health_check(&connection).await?;

        self.client = Some(client);
        self.connection = Some(connection);

        info!("Redis initialization completed successfully");
        Ok(())
    }

    /// Wait for Redis to be ready with exponential backoff
    async fn wait_for_redis(&self) -> Result<()> {
        info!("Waiting for Redis to be ready...");

        let max_attempts = self.config.reconnect_attempts;
        let mut attempt = 1;
        let base_delay = Duration::from_millis(500);

        while attempt <= max_attempts {
            debug!("Redis connection attempt {} of {}", attempt, max_attempts);

            match self.test_connection().await {
                Ok(_) => {
                    info!("Redis is ready!");
                    return Ok(());
                }
                Err(e) => {
                    if attempt == max_attempts {
                        error!("Failed to connect to Redis after {} attempts", max_attempts);
                        return Err(e).context("Redis connection failed");
                    }

                    let delay = base_delay * 2_u32.pow((attempt - 1).min(10));
                    warn!(
                        "Redis not ready (attempt {}): {}. Retrying in {:?}",
                        attempt, e, delay
                    );
                    sleep(delay).await;
                }
            }

            attempt += 1;
        }

        Err(anyhow::anyhow!("Redis connection timeout"))
    }

    /// Test basic Redis connection
    async fn test_connection(&self) -> Result<()> {
        let client =
            Client::open(self.config.url.as_str()).context("Failed to create Redis client")?;

        let mut conn = timeout(
            self.config.connect_timeout,
            client.get_multiplexed_async_connection(),
        )
        .await
        .context("Redis connection timeout")?
        .context("Failed to get Redis connection")?;

        // Test PING command
        let _: String = timeout(
            self.config.response_timeout,
            redis::cmd("PING").query_async(&mut conn),
        )
        .await
        .context("Redis ping timeout")?
        .context("Redis ping failed")?;

        Ok(())
    }

    /// Create Redis client
    fn create_client(&self) -> Result<Client> {
        info!("Creating Redis client...");

        let client =
            Client::open(self.config.url.as_str()).context("Failed to create Redis client")?;

        info!("Redis client created successfully");
        Ok(client)
    }

    /// Create Redis connection
    async fn create_connection(&self, client: &Client) -> Result<MultiplexedConnection> {
        info!("Creating Redis connection...");

        let connection = timeout(
            self.config.connect_timeout,
            client.get_multiplexed_async_connection(),
        )
        .await
        .context("Redis connection timeout")?
        .context("Failed to get Redis connection")?;

        info!("Redis connection established");
        Ok(connection)
    }

    /// Run Redis health check
    pub async fn health_check(&self, connection: &MultiplexedConnection) -> Result<()> {
        debug!("Running Redis health check...");

        let mut conn = connection.clone();

        // Test PING
        let ping_result: String = timeout(
            self.config.response_timeout,
            redis::cmd("PING").query_async(&mut conn),
        )
        .await
        .context("Redis ping timeout")?
        .context("Redis ping failed")?;

        if ping_result != "PONG" {
            return Err(anyhow::anyhow!("Unexpected ping response: {}", ping_result));
        }

        // Get Redis info
        let info: String = timeout(
            self.config.response_timeout,
            redis::cmd("INFO").arg("server").query_async(&mut conn),
        )
        .await
        .context("Redis info timeout")?
        .context("Redis info failed")?;

        // Parse key information
        let mut version = "Unknown".to_string();
        let mut uptime = "Unknown".to_string();
        let mut mode = "Unknown".to_string();

        for line in info.lines() {
            if line.starts_with("redis_version:") {
                version = line.trim_start_matches("redis_version:").to_string();
            } else if line.starts_with("uptime_in_seconds:") {
                if let Ok(seconds) = line.trim_start_matches("uptime_in_seconds:").parse::<u64>() {
                    uptime = format!("{}s", seconds);
                }
            } else if line.starts_with("redis_mode:") {
                mode = line.trim_start_matches("redis_mode:").to_string();
            }
        }

        info!("Redis health check passed:");
        info!("  Version: {}", version);
        info!("  Mode: {}", mode);
        info!("  Uptime: {}", uptime);

        Ok(())
    }

    /// Get Redis connection
    pub fn connection(&self) -> Result<&MultiplexedConnection> {
        self.connection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Redis not initialized"))
    }

    /// Get Redis client
    pub fn client(&self) -> Result<&Client> {
        self.client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Redis not initialized"))
    }

    /// Check if Redis is ready
    pub async fn is_ready(&self) -> bool {
        if let Ok(connection) = self.connection() {
            let mut conn = connection.clone();
            redis::cmd("PING")
                .query_async::<String>(&mut conn)
                .await
                .map(|response| response == "PONG")
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// Get Redis statistics
    pub async fn get_stats(&self) -> Result<RedisStats> {
        let connection = self.connection()?;
        let mut conn = connection.clone();

        // Get memory and client info
        let memory_info: String = redis::cmd("INFO")
            .arg("memory")
            .query_async(&mut conn)
            .await
            .context("Failed to get Redis memory info")?;

        let clients_info: String = redis::cmd("INFO")
            .arg("clients")
            .query_async(&mut conn)
            .await
            .context("Failed to get Redis clients info")?;

        let stats_info: String = redis::cmd("INFO")
            .arg("stats")
            .query_async(&mut conn)
            .await
            .context("Failed to get Redis stats info")?;

        // Parse statistics
        let mut stats = RedisStats::default();

        for line in memory_info
            .lines()
            .chain(clients_info.lines())
            .chain(stats_info.lines())
        {
            if line.starts_with("used_memory:") {
                if let Ok(bytes) = line.trim_start_matches("used_memory:").parse::<u64>() {
                    stats.used_memory = bytes;
                }
            } else if line.starts_with("connected_clients:") {
                if let Ok(clients) = line.trim_start_matches("connected_clients:").parse::<u32>() {
                    stats.connected_clients = clients;
                }
            } else if line.starts_with("total_commands_processed:") {
                if let Ok(commands) = line
                    .trim_start_matches("total_commands_processed:")
                    .parse::<u64>()
                {
                    stats.total_commands = commands;
                }
            } else if line.starts_with("keyspace_hits:") {
                if let Ok(hits) = line.trim_start_matches("keyspace_hits:").parse::<u64>() {
                    stats.keyspace_hits = hits;
                }
            } else if line.starts_with("keyspace_misses:") {
                if let Ok(misses) = line.trim_start_matches("keyspace_misses:").parse::<u64>() {
                    stats.keyspace_misses = misses;
                }
            }
        }

        Ok(stats)
    }

    /// Shutdown Redis connections
    pub async fn shutdown(&mut self) -> Result<()> {
        if self.connection.is_some() {
            info!("Shutting down Redis connections...");
            self.connection.take();
            self.client.take();
            info!("Redis connections closed");
        }
        Ok(())
    }
}

/// Redis statistics
#[derive(Debug, Clone, Default)]
pub struct RedisStats {
    pub used_memory: u64,
    pub connected_clients: u32,
    pub total_commands: u64,
    pub keyspace_hits: u64,
    pub keyspace_misses: u64,
}

impl RedisStats {
    /// Calculate hit ratio
    pub fn hit_ratio(&self) -> f64 {
        let total_requests = self.keyspace_hits + self.keyspace_misses;
        if total_requests > 0 {
            (self.keyspace_hits as f64) / (total_requests as f64) * 100.0
        } else {
            0.0
        }
    }
}

impl std::fmt::Display for RedisStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Memory: {:.2} MB, Clients: {}, Commands: {}, Hit Ratio: {:.1}%",
            self.used_memory as f64 / 1024.0 / 1024.0,
            self.connected_clients,
            self.total_commands,
            self.hit_ratio()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_redis_config_default() {
        let config = RedisConfig::default();
        assert!(!config.url.is_empty());
        assert!(config.max_connections > 0);
        assert!(config.connect_timeout > Duration::ZERO);
    }

    #[tokio::test]
    async fn test_redis_manager_creation() {
        let config = RedisConfig::default();
        let manager = RedisManager::new(config);
        assert!(manager.client.is_none());
        assert!(manager.connection.is_none());
        assert!(!manager.is_ready().await);
    }

    #[tokio::test]
    async fn test_redis_stats_hit_ratio() {
        let stats = RedisStats {
            keyspace_hits: 80,
            keyspace_misses: 20,
            ..Default::default()
        };
        assert_eq!(stats.hit_ratio(), 80.0);

        let empty_stats = RedisStats::default();
        assert_eq!(empty_stats.hit_ratio(), 0.0);
    }

    // Integration tests would require a running Redis instance
    #[ignore]
    #[tokio::test]
    async fn test_redis_initialization() {
        let config = RedisConfig::default();
        let mut manager = RedisManager::new(config);

        // This test requires a running Redis instance
        let result = manager.initialize().await;
        if result.is_ok() {
            assert!(manager.is_ready().await);
            let stats = manager.get_stats().await.unwrap();
            assert!(stats.connected_clients > 0);
            manager.shutdown().await.unwrap();
        }
    }
}
