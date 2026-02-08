//! Retry mechanism for transient failures
//!
//! This module provides configurable retry logic for operations that may fail
//! transiently (network issues, temporary resource unavailability, etc.).
//!
//! Features:
//! - Exponential backoff
//! - Maximum retry attempts
//! - Configurable delays
//! - Jitter to prevent thundering herd

use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts (0 = no retries)
    pub max_attempts: u32,
    /// Initial delay before first retry
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier (typically 2.0 for exponential backoff)
    pub backoff_multiplier: f64,
    /// Add random jitter to delays (prevents thundering herd)
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryPolicy {
    /// Create a policy with no retries
    pub fn none() -> Self {
        Self {
            max_attempts: 0,
            ..Default::default()
        }
    }

    /// Create a policy with a specific number of attempts
    pub fn with_attempts(attempts: u32) -> Self {
        Self {
            max_attempts: attempts,
            ..Default::default()
        }
    }

    /// Create a policy for fast retries (shorter delays)
    pub fn fast() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }

    /// Create a policy for slow retries (longer delays)
    pub fn slow() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }

    /// Calculate delay for a specific attempt
    fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_secs(0);
        }

        // Calculate exponential backoff
        let delay_ms = self.initial_delay.as_millis() as f64
            * self.backoff_multiplier.powi((attempt - 1) as i32);

        let mut delay = Duration::from_millis(delay_ms as u64);

        // Cap at max_delay
        if delay > self.max_delay {
            delay = self.max_delay;
        }

        // Add jitter if enabled (±25% randomness)
        if self.jitter {
            let jitter_factor = 0.75 + (rand::random::<f64>() * 0.5); // 0.75 to 1.25
            let jittered_ms = (delay.as_millis() as f64 * jitter_factor) as u64;
            delay = Duration::from_millis(jittered_ms);
        }

        delay
    }
}

/// Retry a function with the given policy
///
/// The function should return `Result<T, E>` where `E` implements `is_transient()`
/// method to determine if the error can be retried.
///
/// # Example
///
/// ```no_run
/// use video_manager::retry::{retry_with_policy, RetryPolicy};
///
/// async fn flaky_operation() -> Result<String, MyError> {
///     // ... operation that might fail transiently
///     Ok("success".to_string())
/// }
///
/// let policy = RetryPolicy::default();
/// let result = retry_with_policy(policy, "my_operation", || async {
///     flaky_operation().await
/// }).await;
/// ```
pub async fn retry_with_policy<F, Fut, T, E>(
    policy: RetryPolicy,
    operation_name: &str,
    mut f: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display + IsTransient,
{
    let mut attempt = 0;
    let max_attempts = policy.max_attempts + 1; // +1 for initial attempt

    loop {
        attempt += 1;

        debug!(
            "Attempting operation '{}' (attempt {}/{})",
            operation_name, attempt, max_attempts
        );

        match f().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!(
                        "Operation '{}' succeeded after {} attempts",
                        operation_name, attempt
                    );
                }
                return Ok(result);
            }
            Err(e) => {
                // Check if we should retry
                let should_retry = attempt < max_attempts && e.is_transient();

                if !should_retry {
                    if !e.is_transient() {
                        warn!(
                            "Operation '{}' failed with non-transient error after {} attempts: {}",
                            operation_name, attempt, e
                        );
                    } else {
                        warn!(
                            "Operation '{}' failed after {} attempts (max retries exceeded): {}",
                            operation_name, attempt, e
                        );
                    }
                    return Err(e);
                }

                let delay = policy.calculate_delay(attempt);
                warn!(
                    "Operation '{}' failed (attempt {}/{}): {}. Retrying in {:?}",
                    operation_name, attempt, max_attempts, e, delay
                );

                sleep(delay).await;
            }
        }
    }
}

/// Trait for errors that can be classified as transient or permanent
pub trait IsTransient {
    /// Returns true if this error is transient and the operation can be retried
    fn is_transient(&self) -> bool;
}

// Implement for anyhow::Error (default to non-transient)
impl IsTransient for anyhow::Error {
    fn is_transient(&self) -> bool {
        // By default, assume errors are not transient
        // Specific error types should implement this trait
        false
    }
}

// Implement for std::io::Error
impl IsTransient for std::io::Error {
    fn is_transient(&self) -> bool {
        use std::io::ErrorKind;
        matches!(
            self.kind(),
            ErrorKind::ConnectionRefused
                | ErrorKind::ConnectionReset
                | ErrorKind::ConnectionAborted
                | ErrorKind::TimedOut
                | ErrorKind::Interrupted
                | ErrorKind::WouldBlock
        )
    }
}

/// Retry a function with default policy (3 attempts, exponential backoff)
pub async fn retry<F, Fut, T, E>(operation_name: &str, f: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display + IsTransient,
{
    retry_with_policy(RetryPolicy::default(), operation_name, f).await
}

/// Retry a function quickly (3 attempts, short delays)
pub async fn retry_fast<F, Fut, T, E>(operation_name: &str, f: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display + IsTransient,
{
    retry_with_policy(RetryPolicy::fast(), operation_name, f).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[derive(Debug)]
    struct TestError {
        transient: bool,
    }

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "TestError(transient={})", self.transient)
        }
    }

    impl std::error::Error for TestError {}

    impl IsTransient for TestError {
        fn is_transient(&self) -> bool {
            self.transient
        }
    }

    #[tokio::test]
    async fn test_retry_succeeds_first_attempt() {
        let result = retry("test_op", || async { Ok::<_, TestError>(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_succeeds_after_failures() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry("test_op", move || {
            let counter = counter_clone.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(TestError { transient: true })
                } else {
                    Ok(42)
                }
            }
        })
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_fails_non_transient() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry("test_op", move || {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err(TestError { transient: false })
            }
        })
        .await;

        assert!(result.is_err());
        // Should only try once since error is not transient
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_exhausts_attempts() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let policy = RetryPolicy::with_attempts(2);
        let result = retry_with_policy(policy, "test_op", move || {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err(TestError { transient: true })
            }
        })
        .await;

        assert!(result.is_err());
        // Initial attempt + 2 retries = 3 total
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_delay_calculation() {
        let policy = RetryPolicy {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter: false,
        };

        assert_eq!(policy.calculate_delay(0), Duration::from_secs(0));
        assert_eq!(policy.calculate_delay(1), Duration::from_millis(100));
        assert_eq!(policy.calculate_delay(2), Duration::from_millis(200));
        assert_eq!(policy.calculate_delay(3), Duration::from_millis(400));
    }

    #[test]
    fn test_delay_caps_at_max() {
        let policy = RetryPolicy {
            max_attempts: 10,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            jitter: false,
        };

        let delay = policy.calculate_delay(10);
        assert!(delay <= Duration::from_secs(5));
    }
}
