//! Access Control System
//!
//! This crate provides a comprehensive, type-safe access control system for the video server.
//! It implements a 4-layer access model with granular permissions and complete audit logging.
//!
//! # Architecture
//!
//! ## 4-Layer Access Model
//!
//! 1. **Public** - Resources marked as public (read-only)
//! 2. **Access Key** - Temporary access via shareable codes
//! 3. **Group Membership** - Role-based access via group membership
//! 4. **Ownership** - Direct resource ownership (full control)
//!
//! Layers are checked in order, and the highest-priority layer that grants access wins.
//!
//! ## Permission Hierarchy
//!
//! ```text
//! Admin    (5) - Full administrative control
//!   ↓ includes
//! Delete   (4) - Can delete resources
//!   ↓ includes
//! Edit     (3) - Can modify resources
//!   ↓ includes
//! Download (2) - Can download resources
//!   ↓ includes
//! Read     (1) - Can view resources
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use access_control::{AccessControlService, AccessContext, Permission};
//! use common::ResourceType;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;
//! // Create service
//! let service = AccessControlService::new(pool);
//!
//! // Build context
//! let context = AccessContext {
//!     user_id: Some("user123".to_string()),
//!     access_key: None,
//!     resource_type: ResourceType::Video,
//!     resource_id: 42,
//!     ..Default::default()
//! };
//!
//! // Check access
//! let decision = service.check_access(context, Permission::Read).await?;
//!
//! if decision.granted {
//!     println!("Access granted via {:?}: {}", decision.layer, decision.reason);
//! } else {
//!     println!("Access denied: {}", decision.reason);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! - **Type-safe database queries** - No SQL injection vulnerabilities
//! - **Granular permissions** - Read, Download, Edit, Delete, Admin
//! - **Complete audit trail** - Every access decision logged
//! - **Rich error context** - Detailed reasons for access decisions
//! - **Testable** - Mock-friendly architecture
//! - **Performant** - Optimized queries with caching support
//!
//! # Security
//!
//! - All database queries use parameterized statements
//! - Access keys validated for expiration and download limits
//! - Complete audit log for compliance and security monitoring
//! - Rate limiting support for failed access attempts
//! - Privacy-conscious logging (no sensitive data in logs)

pub mod audit;
pub mod error;
pub mod layers;
pub mod models;
pub mod permissions;
pub mod repository;
pub mod service;

// Re-export main types
pub use audit::{AuditLogEntry, AuditLogger};
pub use error::AccessError;
pub use models::{AccessContext, AccessDecision, AccessKeyData, AccessLayer};
pub use permissions::Permission;
pub use repository::AccessRepository;
pub use service::AccessControlService;

// Re-export for convenience
pub use common::{GroupRole, ResourceType};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_exports() {
        // Verify all main types are exported
        let _ = AccessControlService::new;
        let _ = Permission::Read;
        let _ = AccessLayer::Public;
    }
}
