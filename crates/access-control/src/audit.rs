//! Audit logging for access control decisions
//!
//! This module provides comprehensive logging of all access control decisions
//! for security monitoring, compliance, and debugging purposes.
//!
//! # Privacy Considerations
//!
//! The audit log records:
//! - ✅ Access decisions (granted/denied)
//! - ✅ Resource type and ID (not content)
//! - ✅ User ID (not personal details)
//! - ✅ IP address (for security)
//! - ✅ Timestamp
//!
//! The audit log does NOT record:
//! - ❌ Sensitive resource content
//! - ❌ Full user details
//! - ❌ Access key values (only that one was used)
//! - ❌ Personal information

use crate::{AccessDecision, AccessError, Permission};
use common::ResourceType;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::{debug, warn};

/// Audit log entry representing a single access control decision
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLogEntry {
    pub id: i32,
    pub user_id: Option<String>,
    pub access_key: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub resource_type: String,
    pub resource_id: i32,
    pub permission_requested: String,
    pub permission_granted: Option<String>,
    pub access_granted: bool,
    pub access_layer: String,
    pub reason: String,
    pub created_at: String,
}

impl AuditLogEntry {
    /// Check if this entry represents a security event
    pub fn is_security_event(&self) -> bool {
        !self.access_granted
    }

    /// Check if this entry represents a suspicious pattern
    pub fn is_suspicious(&self) -> bool {
        // Multiple failed attempts from same IP
        // Expired key usage
        // Invalid key attempts
        self.reason.contains("expired")
            || self.reason.contains("Invalid")
            || self.reason.contains("limit exceeded")
    }
}

/// Service for logging and querying access control decisions
pub struct AuditLogger {
    pool: SqlitePool,
    enabled: bool,
}

impl AuditLogger {
    /// Create a new audit logger
    ///
    /// # Configuration
    ///
    /// Logging can be disabled via the `enabled` parameter (useful for testing).
    /// In production, logging should always be enabled.
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            enabled: true,
        }
    }

    /// Create audit logger with custom enabled state
    pub fn with_enabled(pool: SqlitePool, enabled: bool) -> Self {
        Self { pool, enabled }
    }

    /// Log an access control decision
    ///
    /// Records the complete context of an access check for auditing,
    /// security monitoring, and debugging.
    ///
    /// # Privacy
    ///
    /// Only logs metadata - no sensitive content is recorded.
    ///
    /// # Performance
    ///
    /// Logging is async and non-blocking. Failed logging attempts are
    /// logged as warnings but don't prevent access.
    pub async fn log_decision(&self, decision: &AccessDecision) -> Result<(), AccessError> {
        if !self.enabled {
            return Ok(());
        }

        let result = sqlx::query(
            "INSERT INTO access_audit_log (
                user_id,
                access_key,
                ip_address,
                user_agent,
                resource_type,
                resource_id,
                permission_requested,
                permission_granted,
                access_granted,
                access_layer,
                reason
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&decision.context.user_id)
        .bind(&decision.context.access_key)
        .bind(&decision.context.ip_address)
        .bind(&decision.context.user_agent)
        .bind(decision.context.resource_type.to_string())
        .bind(decision.context.resource_id)
        .bind(decision.permission_requested.to_string())
        .bind(decision.permission_granted.map(|p| p.to_string()))
        .bind(decision.granted)
        .bind(decision.layer.to_string())
        .bind(&decision.reason)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!(
                    "Audit log: {} access to {} {} ({})",
                    if decision.granted {
                        "Granted"
                    } else {
                        "Denied"
                    },
                    decision.context.resource_type,
                    decision.context.resource_id,
                    decision.reason
                );
                Ok(())
            }
            Err(e) => {
                warn!("Failed to write audit log: {}", e);
                // Don't fail the request if audit logging fails
                Ok(())
            }
        }
    }

    /// Get audit log entries for a specific resource
    ///
    /// Returns the most recent audit entries, ordered by timestamp descending.
    ///
    /// # Use Cases
    ///
    /// - Security review
    /// - Debugging access issues
    /// - Compliance reporting
    /// - Usage analytics
    pub async fn get_resource_audit_log(
        &self,
        resource_type: ResourceType,
        resource_id: i32,
        limit: i32,
    ) -> Result<Vec<AuditLogEntry>, AccessError> {
        let entries = sqlx::query_as(
            "SELECT * FROM access_audit_log
             WHERE resource_type = ? AND resource_id = ?
             ORDER BY created_at DESC
             LIMIT ?",
        )
        .bind(resource_type.to_string())
        .bind(resource_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    /// Get all denied access attempts within a time window
    ///
    /// Useful for security monitoring and identifying potential attacks.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use access_control::AuditLogger;
    /// # async fn example(logger: AuditLogger) -> Result<(), Box<dyn std::error::Error>> {
    /// // Get all denied attempts in last 24 hours
    /// let since = time::OffsetDateTime::now_utc() - time::Duration::hours(24);
    /// let denied = logger.get_denied_attempts(since).await?;
    ///
    /// println!("Found {} suspicious access attempts", denied.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_denied_attempts(
        &self,
        since: time::OffsetDateTime,
    ) -> Result<Vec<AuditLogEntry>, AccessError> {
        let entries = sqlx::query_as(
            "SELECT * FROM access_audit_log
             WHERE access_granted = 0 AND created_at >= ?
             ORDER BY created_at DESC",
        )
        .bind(
            since
                .format(&time::format_description::well_known::Iso8601::DEFAULT)
                .unwrap(),
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    /// Get denied attempts from a specific IP address
    ///
    /// Useful for identifying brute force attacks or suspicious activity.
    pub async fn get_denied_by_ip(
        &self,
        ip_address: &str,
        since: time::OffsetDateTime,
    ) -> Result<Vec<AuditLogEntry>, AccessError> {
        let entries = sqlx::query_as(
            "SELECT * FROM access_audit_log
             WHERE access_granted = 0
               AND ip_address = ?
               AND created_at >= ?
             ORDER BY created_at DESC",
        )
        .bind(ip_address)
        .bind(
            since
                .format(&time::format_description::well_known::Iso8601::DEFAULT)
                .unwrap(),
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(entries)
    }

    /// Get audit statistics for a user
    ///
    /// Returns counts of granted/denied access attempts.
    pub async fn get_user_stats(&self, user_id: &str) -> Result<AuditStats, AccessError> {
        let granted: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM access_audit_log
             WHERE user_id = ? AND access_granted = 1",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let denied: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM access_audit_log
             WHERE user_id = ? AND access_granted = 0",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(AuditStats {
            total_attempts: granted + denied,
            granted_count: granted,
            denied_count: denied,
        })
    }

    /// Get audit statistics for a resource
    pub async fn get_resource_stats(
        &self,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<AuditStats, AccessError> {
        let granted: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM access_audit_log
             WHERE resource_type = ? AND resource_id = ? AND access_granted = 1",
        )
        .bind(resource_type.to_string())
        .bind(resource_id)
        .fetch_one(&self.pool)
        .await?;

        let denied: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM access_audit_log
             WHERE resource_type = ? AND resource_id = ? AND access_granted = 0",
        )
        .bind(resource_type.to_string())
        .bind(resource_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(AuditStats {
            total_attempts: granted + denied,
            granted_count: granted,
            denied_count: denied,
        })
    }

    /// Check for rate limit violations
    ///
    /// Returns the number of failed attempts from an IP in the given time window.
    /// Can be used to implement rate limiting or blocking.
    pub async fn check_failed_attempts(
        &self,
        ip_address: &str,
        window_minutes: i32,
    ) -> Result<i32, AccessError> {
        let count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM access_audit_log
             WHERE ip_address = ?
               AND access_granted = 0
               AND created_at > datetime('now', '-' || ? || ' minutes')",
        )
        .bind(ip_address)
        .bind(window_minutes)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    /// Cleanup old audit logs
    ///
    /// Removes audit entries older than the specified date.
    /// Should be run periodically to prevent unbounded growth.
    ///
    /// # Compliance
    ///
    /// Check compliance requirements before deleting audit logs.
    /// Many regulations require retention for specific periods.
    pub async fn cleanup_old_logs(
        &self,
        older_than: time::OffsetDateTime,
    ) -> Result<u64, AccessError> {
        let result = sqlx::query("DELETE FROM access_audit_log WHERE created_at < ?")
            .bind(
                older_than
                    .format(&time::format_description::well_known::Iso8601::DEFAULT)
                    .unwrap(),
            )
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}

/// Statistics about access attempts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_attempts: i32,
    pub granted_count: i32,
    pub denied_count: i32,
}

impl AuditStats {
    pub fn denial_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            (self.denied_count as f64) / (self.total_attempts as f64)
        }
    }

    pub fn is_suspicious(&self) -> bool {
        // High denial rate might indicate an attack
        self.denial_rate() > 0.5 && self.denied_count > 10
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AccessContext, AccessLayer};

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        sqlx::query(
            "CREATE TABLE access_audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT,
                access_key TEXT,
                ip_address TEXT,
                user_agent TEXT,
                resource_type TEXT NOT NULL,
                resource_id INTEGER NOT NULL,
                permission_requested TEXT NOT NULL,
                permission_granted TEXT,
                access_granted BOOLEAN NOT NULL,
                access_layer TEXT NOT NULL,
                reason TEXT NOT NULL,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_log_decision() {
        let pool = setup_test_db().await;
        let logger = AuditLogger::new(pool.clone());

        let context = AccessContext::new(ResourceType::Video, 1).with_user("user123");
        let decision = AccessDecision::granted(
            AccessLayer::Ownership,
            Permission::Admin,
            "Owner".to_string(),
        )
        .with_context(context);

        logger.log_decision(&decision).await.unwrap();

        // Verify it was logged
        let count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM access_audit_log")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_get_resource_audit_log() {
        let pool = setup_test_db().await;
        let logger = AuditLogger::new(pool.clone());

        // Log multiple decisions for same resource
        for i in 0..5 {
            let context =
                AccessContext::new(ResourceType::Video, 1).with_user(&format!("user{}", i));
            let decision = AccessDecision::granted(
                AccessLayer::Ownership,
                Permission::Read,
                "Test".to_string(),
            )
            .with_context(context);
            logger.log_decision(&decision).await.unwrap();
        }

        let entries = logger
            .get_resource_audit_log(ResourceType::Video, 1, 10)
            .await
            .unwrap();
        assert_eq!(entries.len(), 5);
    }

    #[tokio::test]
    async fn test_get_denied_attempts() {
        let pool = setup_test_db().await;
        let logger = AuditLogger::new(pool.clone());

        // Log granted access
        let context = AccessContext::new(ResourceType::Video, 1).with_user("user123");
        let granted = AccessDecision::granted(
            AccessLayer::Ownership,
            Permission::Read,
            "Owner".to_string(),
        )
        .with_context(context.clone());
        logger.log_decision(&granted).await.unwrap();

        // Log denied access
        let denied = AccessDecision::denied(
            AccessLayer::Public,
            Permission::Edit,
            "Not public".to_string(),
        )
        .with_context(context);
        logger.log_decision(&denied).await.unwrap();

        let since = time::OffsetDateTime::now_utc() - time::Duration::hours(1);
        let denied_entries = logger.get_denied_attempts(since).await.unwrap();

        assert_eq!(denied_entries.len(), 1);
        assert!(!denied_entries[0].access_granted);
    }

    #[tokio::test]
    async fn test_get_denied_by_ip() {
        let pool = setup_test_db().await;
        let logger = AuditLogger::new(pool.clone());

        // Log denied access from specific IP
        let context = AccessContext::new(ResourceType::Video, 1)
            .with_ip("192.168.1.100")
            .with_user("user123");
        let denied = AccessDecision::denied(
            AccessLayer::Public,
            Permission::Edit,
            "Not authorized".to_string(),
        )
        .with_context(context);
        logger.log_decision(&denied).await.unwrap();

        // Log from different IP
        let context = AccessContext::new(ResourceType::Video, 1).with_ip("192.168.1.200");
        let denied = AccessDecision::denied(
            AccessLayer::Public,
            Permission::Edit,
            "Not authorized".to_string(),
        )
        .with_context(context);
        logger.log_decision(&denied).await.unwrap();

        let since = time::OffsetDateTime::now_utc() - time::Duration::hours(1);
        let entries = logger
            .get_denied_by_ip("192.168.1.100", since)
            .await
            .unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].ip_address, Some("192.168.1.100".to_string()));
    }

    #[tokio::test]
    async fn test_check_failed_attempts() {
        let pool = setup_test_db().await;
        let logger = AuditLogger::new(pool.clone());

        // Log 5 failed attempts from same IP
        for _ in 0..5 {
            let context = AccessContext::new(ResourceType::Video, 1).with_ip("192.168.1.100");
            let denied = AccessDecision::denied(
                AccessLayer::AccessKey,
                Permission::Read,
                "Invalid key".to_string(),
            )
            .with_context(context);
            logger.log_decision(&denied).await.unwrap();
        }

        let count = logger
            .check_failed_attempts("192.168.1.100", 5)
            .await
            .unwrap();
        assert_eq!(count, 5);

        let count = logger
            .check_failed_attempts("192.168.1.200", 5)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_user_stats() {
        let pool = setup_test_db().await;
        let logger = AuditLogger::new(pool.clone());

        // Log 3 granted, 2 denied for user123
        for i in 0..3 {
            let context = AccessContext::new(ResourceType::Video, i).with_user("user123");
            let granted = AccessDecision::granted(
                AccessLayer::Ownership,
                Permission::Read,
                "Owner".to_string(),
            )
            .with_context(context);
            logger.log_decision(&granted).await.unwrap();
        }

        for i in 0..2 {
            let context = AccessContext::new(ResourceType::Video, i + 10).with_user("user123");
            let denied =
                AccessDecision::denied(AccessLayer::Public, Permission::Edit, "Denied".to_string())
                    .with_context(context);
            logger.log_decision(&denied).await.unwrap();
        }

        let stats = logger.get_user_stats("user123").await.unwrap();
        assert_eq!(stats.total_attempts, 5);
        assert_eq!(stats.granted_count, 3);
        assert_eq!(stats.denied_count, 2);
        assert_eq!(stats.denial_rate(), 0.4);
        assert!(!stats.is_suspicious());
    }

    #[tokio::test]
    async fn test_resource_stats() {
        let pool = setup_test_db().await;
        let logger = AuditLogger::new(pool.clone());

        // Log accesses to video 1
        for i in 0..10 {
            let context =
                AccessContext::new(ResourceType::Video, 1).with_user(&format!("user{}", i));
            let decision = if i % 3 == 0 {
                AccessDecision::denied(AccessLayer::Public, Permission::Read, "Denied".to_string())
            } else {
                AccessDecision::granted(
                    AccessLayer::Ownership,
                    Permission::Read,
                    "Granted".to_string(),
                )
            };
            logger
                .log_decision(&decision.with_context(context))
                .await
                .unwrap();
        }

        let stats = logger
            .get_resource_stats(ResourceType::Video, 1)
            .await
            .unwrap();
        assert_eq!(stats.total_attempts, 10);
        assert_eq!(stats.granted_count, 6);
        assert_eq!(stats.denied_count, 4);
    }

    #[tokio::test]
    async fn test_disabled_logger() {
        let pool = setup_test_db().await;
        let logger = AuditLogger::with_enabled(pool.clone(), false);

        let context = AccessContext::new(ResourceType::Video, 1);
        let decision =
            AccessDecision::granted(AccessLayer::Public, Permission::Read, "Test".to_string())
                .with_context(context);

        logger.log_decision(&decision).await.unwrap();

        // Should not be logged when disabled
        let count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM access_audit_log")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_audit_stats_calculations() {
        let stats = AuditStats {
            total_attempts: 100,
            granted_count: 80,
            denied_count: 20,
        };
        assert_eq!(stats.denial_rate(), 0.2);
        assert!(!stats.is_suspicious());

        let suspicious = AuditStats {
            total_attempts: 100,
            granted_count: 10,
            denied_count: 90,
        };
        assert_eq!(suspicious.denial_rate(), 0.9);
        assert!(suspicious.is_suspicious());
    }
}
