//! Audit logging for access control decisions
//!
//! This module provides comprehensive logging of all access control decisions
//! for security monitoring, compliance, and debugging purposes.

use crate::{AccessDecision, AccessError};
use common::ResourceType;
use db::access_control::{AuditInsert, AuditRepository};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, warn};

/// Audit log entry representing a single access control decision.
///
/// Mirrors `db::access_control::AuditLogRow` with domain-level methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Check if this entry represents a security event.
    pub fn is_security_event(&self) -> bool {
        !self.access_granted
    }

    /// Check if this entry represents a suspicious pattern.
    pub fn is_suspicious(&self) -> bool {
        self.reason.contains("expired")
            || self.reason.contains("Invalid")
            || self.reason.contains("limit exceeded")
    }
}

impl From<db::access_control::AuditLogRow> for AuditLogEntry {
    fn from(row: db::access_control::AuditLogRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            access_key: row.access_key,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            resource_type: row.resource_type,
            resource_id: row.resource_id,
            permission_requested: row.permission_requested,
            permission_granted: row.permission_granted,
            access_granted: row.access_granted,
            access_layer: row.access_layer,
            reason: row.reason,
            created_at: row.created_at,
        }
    }
}

/// Service for logging and querying access control decisions.
pub struct AuditLogger {
    repo: Arc<dyn AuditRepository>,
    enabled: bool,
}

impl AuditLogger {
    /// Create a new audit logger.
    pub fn new(repo: Arc<dyn AuditRepository>) -> Self {
        Self {
            repo,
            enabled: true,
        }
    }

    /// Create audit logger with custom enabled state.
    pub fn with_enabled(repo: Arc<dyn AuditRepository>, enabled: bool) -> Self {
        Self { repo, enabled }
    }

    /// Log an access control decision.
    pub async fn log_decision(&self, decision: &AccessDecision) -> Result<(), AccessError> {
        if !self.enabled {
            return Ok(());
        }

        let entry = AuditInsert {
            user_id: decision.context.user_id.clone(),
            access_key: decision.context.access_key.clone(),
            ip_address: decision.context.ip_address.clone(),
            user_agent: decision.context.user_agent.clone(),
            resource_type: decision.context.resource_type.to_string(),
            resource_id: decision.context.resource_id,
            permission_requested: decision.permission_requested.to_string(),
            permission_granted: decision.permission_granted.map(|p| p.to_string()),
            access_granted: decision.granted,
            access_layer: decision.layer.to_string(),
            reason: decision.reason.clone(),
        };

        match self.repo.log_entry(&entry).await {
            Ok(_) => {
                debug!(
                    "Audit log: {} access to {} {} ({})",
                    if decision.granted { "Granted" } else { "Denied" },
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

    /// Get audit log entries for a specific resource.
    pub async fn get_resource_audit_log(
        &self,
        resource_type: ResourceType,
        resource_id: i32,
        limit: i32,
    ) -> Result<Vec<AuditLogEntry>, AccessError> {
        let rows = self
            .repo
            .get_resource_audit_log(&resource_type.to_string(), resource_id, limit)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })?;

        Ok(rows.into_iter().map(AuditLogEntry::from).collect())
    }

    /// Get all denied access attempts within a time window.
    pub async fn get_denied_attempts(
        &self,
        since: time::OffsetDateTime,
    ) -> Result<Vec<AuditLogEntry>, AccessError> {
        let since_iso = since
            .format(&time::format_description::well_known::Iso8601::DEFAULT)
            .unwrap();

        let rows = self
            .repo
            .get_denied_attempts(&since_iso)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })?;

        Ok(rows.into_iter().map(AuditLogEntry::from).collect())
    }

    /// Get denied attempts from a specific IP address.
    pub async fn get_denied_by_ip(
        &self,
        ip_address: &str,
        since: time::OffsetDateTime,
    ) -> Result<Vec<AuditLogEntry>, AccessError> {
        let since_iso = since
            .format(&time::format_description::well_known::Iso8601::DEFAULT)
            .unwrap();

        let rows = self
            .repo
            .get_denied_by_ip(ip_address, &since_iso)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })?;

        Ok(rows.into_iter().map(AuditLogEntry::from).collect())
    }

    /// Get audit statistics for a user.
    pub async fn get_user_stats(&self, user_id: &str) -> Result<AuditStats, AccessError> {
        let stats = self
            .repo
            .get_user_stats(user_id)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })?;

        Ok(AuditStats {
            total_attempts: stats.total_attempts,
            granted_count: stats.granted_count,
            denied_count: stats.denied_count,
        })
    }

    /// Get audit statistics for a resource.
    pub async fn get_resource_stats(
        &self,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<AuditStats, AccessError> {
        let stats = self
            .repo
            .get_resource_stats(&resource_type.to_string(), resource_id)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })?;

        Ok(AuditStats {
            total_attempts: stats.total_attempts,
            granted_count: stats.granted_count,
            denied_count: stats.denied_count,
        })
    }

    /// Check for rate limit violations.
    pub async fn check_failed_attempts(
        &self,
        ip_address: &str,
        window_minutes: i32,
    ) -> Result<i32, AccessError> {
        self.repo
            .check_failed_attempts(ip_address, window_minutes)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })
    }

    /// Cleanup old audit logs.
    pub async fn cleanup_old_logs(
        &self,
        older_than: time::OffsetDateTime,
    ) -> Result<u64, AccessError> {
        let iso = older_than
            .format(&time::format_description::well_known::Iso8601::DEFAULT)
            .unwrap();

        self.repo
            .cleanup_old_logs(&iso)
            .await
            .map_err(|e| AccessError::Database {
                message: e.to_string(),
            })
    }
}

/// Statistics about access attempts.
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
        self.denial_rate() > 0.5 && self.denied_count > 10
    }
}
