//! Retry logic with exponential backoff for network operations

use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Configuration for retry logic
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay before first retry
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Exponential backoff factor (e.g., 2.0 for doubling)
    pub backoff_factor: f64,
    /// Add random jitter to prevent thundering herd
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_factor: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    /// Create a retry config for quick operations
    pub fn quick() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(1),
            backoff_factor: 2.0,
            jitter: true,
        }
    }

    /// Create a retry config for network operations
    pub fn network() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(250),
            max_delay: Duration::from_secs(10),
            backoff_factor: 2.0,
            jitter: true,
        }
    }

    /// Create a retry config for API calls
    pub fn api() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_factor: 2.0,
            jitter: true,
        }
    }
}

/// Trait for determining if an error is retryable
pub trait RetryableError {
    /// Check if this error is retryable
    fn is_retryable(&self) -> bool;
}

/// Implement RetryableError for common error types
impl RetryableError for reqwest::Error {
    fn is_retryable(&self) -> bool {
        // Retry on network errors, timeouts, and 5xx status codes
        if self.is_timeout() || self.is_connect() {
            return true;
        }
        
        if let Some(status) = self.status() {
            return status.is_server_error() || status.as_u16() == 429; // Rate limit
        }
        
        false
    }
}

impl RetryableError for std::io::Error {
    fn is_retryable(&self) -> bool {
        use std::io::ErrorKind;
        matches!(
            self.kind(),
            ErrorKind::ConnectionRefused
                | ErrorKind::ConnectionReset
                | ErrorKind::ConnectionAborted
                | ErrorKind::TimedOut
                | ErrorKind::Interrupted
        )
    }
}

/// Execute an operation with retry logic
pub async fn retry_with_config<F, T, E>(
    config: &RetryConfig,
    operation_name: &str,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> futures::future::BoxFuture<'static, Result<T, E>>,
    E: RetryableError + std::fmt::Display,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay;

    loop {
        attempt += 1;
        
        debug!(
            "Attempting {} (attempt {}/{})",
            operation_name, attempt, config.max_attempts
        );

        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!(
                        "{} succeeded after {} attempts",
                        operation_name, attempt
                    );
                }
                return Ok(result);
            }
            Err(error) => {
                if !error.is_retryable() {
                    debug!("{} failed with non-retryable error: {}", operation_name, error);
                    return Err(error);
                }

                if attempt >= config.max_attempts {
                    warn!(
                        "{} failed after {} attempts: {}",
                        operation_name, config.max_attempts, error
                    );
                    return Err(error);
                }

                // Calculate next delay with exponential backoff
                let mut next_delay = delay.as_secs_f64() * config.backoff_factor;
                
                // Add jitter if enabled
                if config.jitter {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    let jitter_factor = rng.gen_range(0.8..1.2);
                    next_delay *= jitter_factor;
                }
                
                // Cap at max delay
                next_delay = next_delay.min(config.max_delay.as_secs_f64());
                delay = Duration::from_secs_f64(next_delay);

                warn!(
                    "{} failed (attempt {}/{}), retrying in {:?}: {}",
                    operation_name, attempt, config.max_attempts, delay, error
                );

                sleep(delay).await;
            }
        }
    }
}

/// Convenience function for retry with default config
pub async fn retry<F, T, E>(
    operation_name: &str,
    operation: F,
) -> Result<T, E>
where
    F: FnMut() -> futures::future::BoxFuture<'static, Result<T, E>>,
    E: RetryableError + std::fmt::Display,
{
    retry_with_config(&RetryConfig::default(), operation_name, operation).await
}

/// Extension trait for adding retry to futures
pub trait RetryExt: Sized {
    type Output;
    type Error: RetryableError;

    /// Retry this operation with default config
    async fn retry(self, operation_name: &str) -> Result<Self::Output, Self::Error>;

    /// Retry this operation with custom config
    async fn retry_with_config(
        self,
        operation_name: &str,
        config: &RetryConfig,
    ) -> Result<Self::Output, Self::Error>;
}

/// Macro for easily adding retry to async operations
#[macro_export]
macro_rules! with_retry {
    ($operation_name:expr, $operation:expr) => {
        $crate::retry::retry($operation_name, || Box::pin($operation))
    };
    ($operation_name:expr, $config:expr, $operation:expr) => {
        $crate::retry::retry_with_config($config, $operation_name, || Box::pin($operation))
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[derive(Debug)]
    struct TestError {
        retryable: bool,
    }

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "TestError(retryable: {})", self.retryable)
        }
    }

    impl RetryableError for TestError {
        fn is_retryable(&self) -> bool {
            self.retryable
        }
    }

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let result = retry("test_operation", move || {
            let attempts = attempts_clone.clone();
            Box::pin(async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Ok::<_, TestError>(42)
            })
        })
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_factor: 2.0,
            jitter: false,
        };

        let result = retry_with_config(&config, "test_operation", move || {
            let attempts = attempts_clone.clone();
            Box::pin(async move {
                let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                if attempt < 2 {
                    Err(TestError { retryable: true })
                } else {
                    Ok(42)
                }
            })
        })
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_non_retryable_error() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let result = retry("test_operation", move || {
            let attempts = attempts_clone.clone();
            Box::pin(async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<i32, _>(TestError { retryable: false })
            })
        })
        .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }
}