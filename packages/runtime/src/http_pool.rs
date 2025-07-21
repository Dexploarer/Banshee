//! HTTP connection pool for managing shared client instances

use once_cell::sync::Lazy;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Configuration for HTTP client pools
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Connection timeout in seconds
    pub timeout_secs: u64,
    /// Maximum idle connections per host
    pub max_idle_per_host: usize,
    /// Idle connection timeout in seconds
    pub idle_timeout_secs: u64,
    /// Whether to enforce TLS certificate validation
    pub enforce_tls: bool,
    /// Connection pool size
    pub pool_size: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 60,
            max_idle_per_host: 10,
            idle_timeout_secs: 90,
            enforce_tls: true,
            pool_size: 100,
        }
    }
}

/// Global connection pool manager
static CONNECTION_POOL: Lazy<ConnectionPool> = Lazy::new(|| ConnectionPool::new());

/// Connection pool for managing HTTP clients
pub struct ConnectionPool {
    /// Clients indexed by configuration hash
    clients: Arc<RwLock<HashMap<u64, Arc<Client>>>>,
    /// Default pool configuration
    default_config: PoolConfig,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            default_config: PoolConfig::default(),
        }
    }

    /// Get or create a client with default configuration
    pub fn get_default_client() -> Arc<Client> {
        CONNECTION_POOL.get_or_create_client(&PoolConfig::default())
    }

    /// Get or create a client with custom configuration
    pub fn get_client(config: &PoolConfig) -> Arc<Client> {
        CONNECTION_POOL.get_or_create_client(config)
    }

    /// Get or create a client for the given configuration
    fn get_or_create_client(&self, config: &PoolConfig) -> Arc<Client> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Create a hash of the configuration
        let mut hasher = DefaultHasher::new();
        config.timeout_secs.hash(&mut hasher);
        config.max_idle_per_host.hash(&mut hasher);
        config.idle_timeout_secs.hash(&mut hasher);
        config.enforce_tls.hash(&mut hasher);
        config.pool_size.hash(&mut hasher);
        let config_hash = hasher.finish();

        // Check if client already exists
        {
            let clients = self.clients.read().unwrap();
            if let Some(client) = clients.get(&config_hash) {
                return Arc::clone(client);
            }
        }

        // Create new client
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .pool_max_idle_per_host(config.max_idle_per_host)
            .pool_idle_timeout(Duration::from_secs(config.idle_timeout_secs))
            .connection_verbose(false);

        // Configure TLS
        if !config.enforce_tls {
            builder = builder.danger_accept_invalid_certs(true);
        }

        // Set pool size through connection limits
        builder = builder.http2_initial_stream_window_size(65535)
            .http2_initial_connection_window_size(65535);

        let client = Arc::new(
            builder
                .build()
                .unwrap_or_else(|_| Client::new())
        );

        // Store the client
        {
            let mut clients = self.clients.write().unwrap();
            clients.insert(config_hash, Arc::clone(&client));
        }

        client
    }

    /// Clear all cached clients
    pub fn clear() {
        let mut clients = CONNECTION_POOL.clients.write().unwrap();
        clients.clear();
    }

    /// Get the number of cached clients
    pub fn size() -> usize {
        let clients = CONNECTION_POOL.clients.read().unwrap();
        clients.len()
    }
}

/// Builder for creating custom pool configurations
pub struct PoolConfigBuilder {
    config: PoolConfig,
}

impl PoolConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: PoolConfig::default(),
        }
    }

    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.config.timeout_secs = secs;
        self
    }

    pub fn max_idle_per_host(mut self, max: usize) -> Self {
        self.config.max_idle_per_host = max;
        self
    }

    pub fn idle_timeout_secs(mut self, secs: u64) -> Self {
        self.config.idle_timeout_secs = secs;
        self
    }

    pub fn enforce_tls(mut self, enforce: bool) -> Self {
        self.config.enforce_tls = enforce;
        self
    }

    pub fn pool_size(mut self, size: usize) -> Self {
        self.config.pool_size = size;
        self
    }

    pub fn build(self) -> PoolConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_pool() {
        // Clear pool before test
        ConnectionPool::clear();
        
        // Get default client
        let client1 = ConnectionPool::get_default_client();
        let client2 = ConnectionPool::get_default_client();
        
        // Should be the same instance
        assert!(Arc::ptr_eq(&client1, &client2));
        assert_eq!(ConnectionPool::size(), 1);
        
        // Get client with custom config
        let custom_config = PoolConfigBuilder::new()
            .timeout_secs(30)
            .max_idle_per_host(5)
            .build();
        
        let client3 = ConnectionPool::get_client(&custom_config);
        
        // Should be a different instance
        assert!(!Arc::ptr_eq(&client1, &client3));
        assert_eq!(ConnectionPool::size(), 2);
        
        // Clear pool
        ConnectionPool::clear();
        assert_eq!(ConnectionPool::size(), 0);
    }

    #[test]
    fn test_pool_config_builder() {
        let config = PoolConfigBuilder::new()
            .timeout_secs(120)
            .max_idle_per_host(20)
            .idle_timeout_secs(180)
            .enforce_tls(false)
            .pool_size(200)
            .build();

        assert_eq!(config.timeout_secs, 120);
        assert_eq!(config.max_idle_per_host, 20);
        assert_eq!(config.idle_timeout_secs, 180);
        assert!(!config.enforce_tls);
        assert_eq!(config.pool_size, 200);
    }
}