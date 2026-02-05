# Access Control System Refactor - Design Document

**Branch:** `feature/refined-masterplan`  
**Status:** 🎨 Design Phase  
**Date:** February 2026  
**Author:** System Architecture Team

---

## 📋 Table of Contents

1. [Problem Statement](#-problem-statement)
2. [Current System Analysis](#-current-system-analysis)
3. [Proposed Architecture](#-proposed-architecture)
4. [Implementation Plan](#-implementation-plan)
5. [Migration Strategy](#-migration-strategy)
6. [Testing Strategy](#-testing-strategy)
7. [Success Metrics](#-success-metrics)

---

## 🎯 Problem Statement

### Current Issues

1. **SQL Injection Risk**
   - String concatenation in queries (`format!("SELECT ... FROM {}", table)`)
   - No type-safe query building
   - Vulnerable to table name manipulation

2. **Mixed Concerns**
   - Access logic scattered across multiple functions
   - No single source of truth for permissions
   - Hard to test and maintain

3. **Inconsistent Permission Model**
   - Boolean yes/no access only
   - No granular permissions (read vs download vs edit)
   - Can't distinguish between view-only and full access

4. **Trait Not Used**
   - `AccessControl` trait defined but not implemented
   - No polymorphic access checking
   - Can't swap implementations for testing

5. **Unclear Layer Model**
   - Comments mention 4 layers but implementation differs
   - Layer numbering inconsistent
   - Hard to understand access flow

6. **No Audit Trail**
   - Limited logging of access decisions
   - Hard to debug "why was access denied?"
   - No compliance/security audit support

### Goals of Refactor

✅ **Security:** Type-safe queries, no SQL injection risk  
✅ **Clarity:** Single, well-defined access control service  
✅ **Flexibility:** Granular permissions (read, download, edit, delete, admin)  
✅ **Testability:** Mock-able, unit testable, clear interfaces  
✅ **Auditability:** Complete access decision logging  
✅ **Maintainability:** Easy to extend for new resource types  

---

## 🔍 Current System Analysis

### Current File: `crates/common/src/access_control.rs`

**Functions:**
- `check_resource_access()` - Main entry point
- `is_public()` - Check public visibility
- `is_owner()` - Check ownership
- `check_group_membership_access()` - Check group membership
- `check_access_key_permission()` - Check access key
- `check_resource_in_group()` - Check if resource in group
- `log_access_key_usage()` - Log key usage

**Problems Identified:**

```rust
// ❌ PROBLEM 1: SQL Injection Risk
let table = match resource_type {
    ResourceType::Video => "videos",
    ResourceType::Image => "images",
    // ...
};
let query = format!("SELECT is_public FROM {} WHERE id = ?", table);
// String formatting with table names is dangerous!

// ❌ PROBLEM 2: Boolean-only permissions
pub async fn check_resource_access(...) -> Result<bool, Error>
// Can only return true/false, no permission levels

// ❌ PROBLEM 3: Nested logic hard to follow
if is_public(...) { return Ok(true); }
if let Some(key) = access_key { ... }
let user = user_id.ok_or(...)?;
if is_owner(...) { return Ok(true); }
check_group_membership_access(...)
// Flow is complex and error-prone

// ❌ PROBLEM 4: Trait not actually used
pub trait AccessControl { ... }
// Defined but no impl blocks anywhere
```

### Current Trait: `crates/common/src/traits.rs`

```rust
#[async_trait]
pub trait AccessControl {
    async fn check_access(...) -> Result<bool, Error>;
    async fn check_group_access(...) -> Result<bool, Error>;
    async fn check_key_access(...) -> Result<bool, Error>;
}
```

**Problems:**
- Not implemented by any struct
- Methods too generic
- Missing permission parameter
- No context about WHY access was granted/denied

---

## 🏗️ Proposed Architecture

### Core Principles

1. **Single Responsibility:** One service, one job
2. **Type Safety:** No string concatenation in SQL
3. **Explicit Permissions:** Clear permission levels
4. **Audit Trail:** Log all access decisions
5. **Testability:** Easy to mock and test

### New Structure

```
crates/access-control/              # NEW CRATE
├── src/
│   ├── lib.rs                      # Public exports
│   ├── service.rs                  # AccessControlService
│   ├── models.rs                   # AccessDecision, AccessContext
│   ├── permissions.rs              # Permission enum and logic
│   ├── layers/                     # 4-layer implementation
│   │   ├── mod.rs
│   │   ├── public.rs               # Layer 1: Public access
│   │   ├── access_key.rs           # Layer 2: Access keys
│   │   ├── group.rs                # Layer 3: Group membership
│   │   └── owner.rs                # Layer 4: Ownership
│   ├── repository.rs               # Database queries (type-safe)
│   ├── audit.rs                    # Audit logging
│   └── error.rs                    # Access-specific errors
├── Cargo.toml
└── README.md
```

---

## 📐 New Data Models

### AccessDecision

```rust
/// Result of an access check with full context
#[derive(Debug, Clone)]
pub struct AccessDecision {
    /// Was access granted?
    pub granted: bool,
    
    /// Which layer granted/denied access?
    pub layer: AccessLayer,
    
    /// What permission was requested?
    pub permission_requested: Permission,
    
    /// What permission was granted (may be less than requested)?
    pub permission_granted: Option<Permission>,
    
    /// Why was this decision made?
    pub reason: String,
    
    /// Additional context for auditing
    pub context: AccessContext,
}

impl AccessDecision {
    pub fn granted(layer: AccessLayer, permission: Permission, reason: String) -> Self {
        Self {
            granted: true,
            layer,
            permission_requested: permission,
            permission_granted: Some(permission),
            reason,
            context: AccessContext::default(),
        }
    }
    
    pub fn denied(layer: AccessLayer, permission: Permission, reason: String) -> Self {
        Self {
            granted: false,
            layer,
            permission_requested: permission,
            permission_granted: None,
            reason,
            context: AccessContext::default(),
        }
    }
}
```

### AccessLayer

```rust
/// The 4 layers of access control
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessLayer {
    /// Layer 1: Resource is public (anyone can access)
    Public,
    
    /// Layer 2: Access via access key/code
    AccessKey,
    
    /// Layer 3: Access via group membership
    GroupMembership,
    
    /// Layer 4: Direct ownership
    Ownership,
}

impl AccessLayer {
    /// Priority order (higher number = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            Self::Public => 1,
            Self::AccessKey => 2,
            Self::GroupMembership => 3,
            Self::Ownership => 4,
        }
    }
}
```

### AccessContext

```rust
/// Context information for access decisions
#[derive(Debug, Clone, Default)]
pub struct AccessContext {
    /// User ID (if authenticated)
    pub user_id: Option<String>,
    
    /// Access key (if provided)
    pub access_key: Option<String>,
    
    /// Resource being accessed
    pub resource_type: ResourceType,
    pub resource_id: i32,
    
    /// Request metadata
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub referer: Option<String>,
    
    /// Timestamp of decision
    pub timestamp: time::OffsetDateTime,
}
```

### Permission Enum (Enhanced)

```rust
/// Granular permission levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Permission {
    /// Can view/read the resource
    Read = 1,
    
    /// Can view and download the resource
    Download = 2,
    
    /// Can view, download, and modify the resource
    Edit = 3,
    
    /// Can view, download, modify, and delete the resource
    Delete = 4,
    
    /// Full administrative control
    Admin = 5,
}

impl Permission {
    /// Check if this permission includes another
    pub fn includes(&self, other: Permission) -> bool {
        *self as u8 >= other as u8
    }
    
    /// Get all permissions included by this level
    pub fn included_permissions(&self) -> Vec<Permission> {
        let level = *self as u8;
        vec![
            Permission::Read,
            Permission::Download,
            Permission::Edit,
            Permission::Delete,
            Permission::Admin,
        ]
        .into_iter()
        .filter(|p| (*p as u8) <= level)
        .collect()
    }
}

// Examples:
// Permission::Edit.includes(Permission::Read) => true
// Permission::Read.includes(Permission::Edit) => false
// Permission::Admin.includes(Permission::Delete) => true
```

---

## 🏛️ New AccessControlService

### Service Structure

```rust
/// Main access control service
pub struct AccessControlService {
    pool: SqlitePool,
    repository: AccessRepository,
    audit_logger: AuditLogger,
}

impl AccessControlService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            repository: AccessRepository::new(pool.clone()),
            audit_logger: AuditLogger::new(pool.clone()),
            pool,
        }
    }
    
    /// Check if access is granted for a resource
    pub async fn check_access(
        &self,
        context: AccessContext,
        required_permission: Permission,
    ) -> Result<AccessDecision, Error> {
        // Check all layers in order (1-4)
        let layers = vec![
            self.check_layer_1_public(&context, required_permission).await?,
            self.check_layer_2_access_key(&context, required_permission).await?,
            self.check_layer_3_group_membership(&context, required_permission).await?,
            self.check_layer_4_ownership(&context, required_permission).await?,
        ];
        
        // Find the highest priority layer that grants access
        let decision = layers
            .into_iter()
            .filter(|d| d.granted)
            .max_by_key(|d| d.layer.priority())
            .unwrap_or_else(|| AccessDecision::denied(
                AccessLayer::Public,
                required_permission,
                "No access layer granted permission".to_string(),
            ));
        
        // Log the decision
        self.audit_logger.log_decision(&decision).await?;
        
        Ok(decision)
    }
    
    /// Layer 1: Check if resource is public
    async fn check_layer_1_public(
        &self,
        context: &AccessContext,
        permission: Permission,
    ) -> Result<AccessDecision, Error> {
        let is_public = self.repository
            .is_resource_public(context.resource_type, context.resource_id)
            .await?;
        
        if is_public && permission <= Permission::Read {
            Ok(AccessDecision::granted(
                AccessLayer::Public,
                Permission::Read,
                "Resource is publicly accessible".to_string(),
            ))
        } else {
            Ok(AccessDecision::denied(
                AccessLayer::Public,
                permission,
                "Resource is not public or requires higher permission".to_string(),
            ))
        }
    }
    
    /// Layer 2: Check access via access key
    async fn check_layer_2_access_key(
        &self,
        context: &AccessContext,
        permission: Permission,
    ) -> Result<AccessDecision, Error> {
        let Some(key) = &context.access_key else {
            return Ok(AccessDecision::denied(
                AccessLayer::AccessKey,
                permission,
                "No access key provided".to_string(),
            ));
        };
        
        let key_data = self.repository
            .get_access_key_data(key)
            .await?;
        
        let Some(key_data) = key_data else {
            return Ok(AccessDecision::denied(
                AccessLayer::AccessKey,
                permission,
                "Invalid access key".to_string(),
            ));
        };
        
        // Check expiration
        if key_data.is_expired() {
            return Ok(AccessDecision::denied(
                AccessLayer::AccessKey,
                permission,
                "Access key has expired".to_string(),
            ));
        }
        
        // Check download limit
        if key_data.is_limit_exceeded() {
            return Ok(AccessDecision::denied(
                AccessLayer::AccessKey,
                permission,
                "Access key download limit exceeded".to_string(),
            ));
        }
        
        // Check if key grants access to this resource
        let has_access = self.repository
            .access_key_grants_resource(
                &key_data,
                context.resource_type,
                context.resource_id,
            )
            .await?;
        
        if has_access && permission <= key_data.permission_level {
            Ok(AccessDecision::granted(
                AccessLayer::AccessKey,
                key_data.permission_level,
                format!("Access granted via key: {}", key_data.description),
            ))
        } else {
            Ok(AccessDecision::denied(
                AccessLayer::AccessKey,
                permission,
                "Access key does not grant sufficient permission".to_string(),
            ))
        }
    }
    
    /// Layer 3: Check access via group membership
    async fn check_layer_3_group_membership(
        &self,
        context: &AccessContext,
        permission: Permission,
    ) -> Result<AccessDecision, Error> {
        let Some(user_id) = &context.user_id else {
            return Ok(AccessDecision::denied(
                AccessLayer::GroupMembership,
                permission,
                "Not authenticated".to_string(),
            ));
        };
        
        // Get resource's group
        let group_id = self.repository
            .get_resource_group(context.resource_type, context.resource_id)
            .await?;
        
        let Some(group_id) = group_id else {
            return Ok(AccessDecision::denied(
                AccessLayer::GroupMembership,
                permission,
                "Resource not in any group".to_string(),
            ));
        };
        
        // Check user's role in group
        let role = self.repository
            .get_user_group_role(user_id, group_id)
            .await?;
        
        let Some(role) = role else {
            return Ok(AccessDecision::denied(
                AccessLayer::GroupMembership,
                permission,
                "User is not a member of the resource's group".to_string(),
            ));
        };
        
        // Map role to permission
        let granted_permission = role.to_permission();
        
        if granted_permission >= permission {
            Ok(AccessDecision::granted(
                AccessLayer::GroupMembership,
                granted_permission,
                format!("Access granted via group membership (role: {:?})", role),
            ))
        } else {
            Ok(AccessDecision::denied(
                AccessLayer::GroupMembership,
                permission,
                format!("Group role {:?} insufficient for {:?}", role, permission),
            ))
        }
    }
    
    /// Layer 4: Check direct ownership
    async fn check_layer_4_ownership(
        &self,
        context: &AccessContext,
        permission: Permission,
    ) -> Result<AccessDecision, Error> {
        let Some(user_id) = &context.user_id else {
            return Ok(AccessDecision::denied(
                AccessLayer::Ownership,
                permission,
                "Not authenticated".to_string(),
            ));
        };
        
        let is_owner = self.repository
            .is_resource_owner(user_id, context.resource_type, context.resource_id)
            .await?;
        
        if is_owner {
            Ok(AccessDecision::granted(
                AccessLayer::Ownership,
                Permission::Admin,
                "Owner has full administrative access".to_string(),
            ))
        } else {
            Ok(AccessDecision::denied(
                AccessLayer::Ownership,
                permission,
                "User is not the owner".to_string(),
            ))
        }
    }
}
```

---

## 🎨 Permission System Design

### GroupRole → Permission Mapping

```rust
impl GroupRole {
    /// Convert group role to permission level
    pub fn to_permission(&self) -> Permission {
        match self {
            GroupRole::Owner => Permission::Admin,
            GroupRole::Admin => Permission::Admin,
            GroupRole::Editor => Permission::Edit,
            GroupRole::Contributor => Permission::Download,
            GroupRole::Viewer => Permission::Read,
        }
    }
}
```

### Permission Hierarchy

```
Permission::Admin (5)
    ↓ includes
Permission::Delete (4)
    ↓ includes
Permission::Edit (3)
    ↓ includes
Permission::Download (2)
    ↓ includes
Permission::Read (1)
```

**Examples:**
- `Permission::Admin` includes all lower permissions
- `Permission::Edit` includes Download and Read
- `Permission::Read` only includes Read

### Access Level by Layer

| Layer | Max Permission | Notes |
|-------|---------------|-------|
| **Public** | Read | View-only, no download |
| **Access Key** | Varies | Configurable per key |
| **Group Membership** | Varies | Based on role |
| **Ownership** | Admin | Full control |

---

## 🗄️ Database Changes

### New Tables

#### 1. Access Audit Log

```sql
CREATE TABLE access_audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    
    -- Request context
    user_id TEXT,
    access_key TEXT,
    ip_address TEXT,
    user_agent TEXT,
    
    -- Resource accessed
    resource_type TEXT NOT NULL,
    resource_id INTEGER NOT NULL,
    
    -- Permission check
    permission_requested TEXT NOT NULL,
    permission_granted TEXT,
    
    -- Decision
    access_granted BOOLEAN NOT NULL,
    access_layer TEXT NOT NULL,
    reason TEXT NOT NULL,
    
    -- Metadata
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Indexes
    INDEX idx_audit_user (user_id),
    INDEX idx_audit_resource (resource_type, resource_id),
    INDEX idx_audit_timestamp (created_at),
    INDEX idx_audit_denied (access_granted) WHERE access_granted = 0
);
```

#### 2. Enhanced Access Keys Table

```sql
-- Update access_keys table to include permission level
ALTER TABLE access_keys ADD COLUMN permission_level TEXT NOT NULL DEFAULT 'read';
-- Values: 'read', 'download', 'edit', 'delete', 'admin'

-- Add check constraint
ALTER TABLE access_keys ADD CONSTRAINT check_permission_level 
    CHECK (permission_level IN ('read', 'download', 'edit', 'delete', 'admin'));
```

### Migration Script

```sql
-- Migration: 004_access_control_refactor.sql

-- 1. Create audit log table
CREATE TABLE access_audit_log (
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
);

CREATE INDEX idx_audit_user ON access_audit_log(user_id);
CREATE INDEX idx_audit_resource ON access_audit_log(resource_type, resource_id);
CREATE INDEX idx_audit_timestamp ON access_audit_log(created_at);
CREATE INDEX idx_audit_denied ON access_audit_log(access_granted) WHERE access_granted = 0;

-- 2. Add permission_level to access_keys
ALTER TABLE access_keys ADD COLUMN permission_level TEXT NOT NULL DEFAULT 'read';

-- 3. Migrate existing keys (default to 'download' for backwards compatibility)
UPDATE access_keys SET permission_level = 'download';

-- 4. Add check constraint (SQLite doesn't support ALTER TABLE ... ADD CONSTRAINT)
-- Will be enforced in application code

-- 5. Add description to access_keys if not exists (for better auditing)
-- Already exists, no change needed
```

---

## 🔧 Repository Layer (Type-Safe)

### AccessRepository

```rust
/// Type-safe database queries for access control
pub struct AccessRepository {
    pool: SqlitePool,
}

impl AccessRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    
    /// Check if resource is public (type-safe, no string concat)
    pub async fn is_resource_public(
        &self,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<bool, Error> {
        match resource_type {
            ResourceType::Video => {
                sqlx::query_scalar("SELECT is_public FROM videos WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
                    .ok_or(Error::NotFound)
            }
            ResourceType::Image => {
                sqlx::query_scalar("SELECT is_public FROM images WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
                    .ok_or(Error::NotFound)
            }
            ResourceType::File => {
                sqlx::query_scalar("SELECT is_public FROM files WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
                    .ok_or(Error::NotFound)
            }
            ResourceType::Folder => {
                sqlx::query_scalar("SELECT is_public FROM folders WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
                    .ok_or(Error::NotFound)
            }
        }
    }
    
    /// Check if user owns resource
    pub async fn is_resource_owner(
        &self,
        user_id: &str,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<bool, Error> {
        let owner: Option<String> = match resource_type {
            ResourceType::Video => {
                sqlx::query_scalar("SELECT user_id FROM videos WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
            ResourceType::Image => {
                sqlx::query_scalar("SELECT user_id FROM images WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
            ResourceType::File => {
                sqlx::query_scalar("SELECT user_id FROM files WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
            ResourceType::Folder => {
                sqlx::query_scalar("SELECT user_id FROM folders WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await?
            }
        };
        
        Ok(owner.as_deref() == Some(user_id))
    }
    
    /// Get resource's group ID
    pub async fn get_resource_group(
        &self,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<Option<i32>, Error> {
        match resource_type {
            ResourceType::Video => {
                sqlx::query_scalar("SELECT group_id FROM videos WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(Into::into)
            }
            ResourceType::Image => {
                sqlx::query_scalar("SELECT group_id FROM images WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(Into::into)
            }
            ResourceType::File => {
                sqlx::query_scalar("SELECT group_id FROM files WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(Into::into)
            }
            ResourceType::Folder => {
                sqlx::query_scalar("SELECT group_id FROM folders WHERE id = ?")
                    .bind(resource_id)
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    
    /// Get user's role in a group
    pub async fn get_user_group_role(
        &self,
        user_id: &str,
        group_id: i32,
    ) -> Result<Option<GroupRole>, Error> {
        let role: Option<String> = sqlx::query_scalar(
            "SELECT role FROM group_members WHERE user_id = ? AND group_id = ?"
        )
        .bind(user_id)
        .bind(group_id)
        .fetch_optional(&self.pool)
        .await?;
        
        role.map(|r| r.parse()).transpose()
    }
    
    /// Get access key data with permission level
    pub async fn get_access_key_data(
        &self,
        key: &str,
    ) -> Result<Option<AccessKeyData>, Error> {
        sqlx::query_as(
            "SELECT 
                id, 
                key, 
                description,
                permission_level,
                access_group_id,
                share_all_group_resources,
                expires_at,
                max_downloads,
                current_downloads,
                is_active
             FROM access_keys 
             WHERE key = ? AND is_active = 1"
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }
    
    /// Check if access key grants access to specific resource
    pub async fn access_key_grants_resource(
        &self,
        key_data: &AccessKeyData,
        resource_type: ResourceType,
        resource_id: i32,
    ) -> Result<bool, Error> {
        // If share_all_group_resources is true, check if resource is in the group
        if key_data.share_all_group_resources {
            if let Some(group_id) = key_data.access_group_id {
                let res_group = self.get_resource_group(resource_type, resource_id).await?;
                return Ok(res_group == Some(group_id));
            }
        }
        
        // Otherwise check specific permissions
        let has_permission: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM access_key_permissions
                WHERE access_key_id = ? 
                  AND resource_type = ? 
                  AND resource_id = ?
            )"
        )
        .bind(key_data.id)
        .bind(resource_type.to_string())
        .bind(resource_id)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(has_permission)
    }
}
```

### AccessKeyData

```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccessKeyData {
    pub id: i32,
    pub key: String,
    pub description: String,
    pub permission_level: Permission,
    pub access_group_id: Option<i32>,
    pub share_all_group_resources: bool,
    pub expires_at: Option<String>,
    pub max_downloads: Option<i32>,
    pub current_downloads: i32,
    pub is_active: bool,
}

impl AccessKeyData {
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = &self.expires_at {
            if let Ok(expires) = time::OffsetDateTime::parse(
                exp,
                &time::format_description::well_known::Iso8601::DEFAULT,
            ) {
                return expires < time::OffsetDateTime::now_utc();
            }
        }
        false
    }
    
    pub fn is_limit_exceeded(&self) -> bool {
        if let Some(max) = self.max_downloads {
            return self.current_downloads >= max;
        }
        false
    }
}
```

---

## 🔍 Audit Logging

### AuditLogger

```rust
/// Logs all access control decisions for security and compliance
pub struct AuditLogger {
    pool: SqlitePool,
    enabled: bool,
}

impl AuditLogger {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            enabled: true, // Can be configured via env var
        }
    }
    
    /// Log an access decision
    pub async fn log_decision(&self, decision: &AccessDecision) -> Result<(), Error> {
        if !self.enabled {
            return Ok(());
        }
        
        sqlx::query(
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
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
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
        .await?;
        
        Ok(())
    }
    
    /// Get audit log for a resource
    pub async fn get_resource_audit_log(
        &self,
        resource_type: ResourceType,
        resource_id: i32,
        limit: i32,
    ) -> Result<Vec<AuditLogEntry>, Error> {
        sqlx::query_as(
            "SELECT * FROM access_audit_log
             WHERE resource_type = ? AND resource_id = ?
             ORDER BY created_at DESC
             LIMIT ?"
        )
        .bind(resource_type.to_string())
        .bind(resource_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }
    
    /// Get denied access attempts (security monitoring)
    pub async fn get_denied_attempts(
        &self,
        since: time::OffsetDateTime,
    ) -> Result<Vec<AuditLogEntry>, Error> {
        sqlx::query_as(
            "SELECT * FROM access_audit_log
             WHERE access_granted = 0 AND created_at >= ?
             ORDER BY created_at DESC"
        )
        .bind(since.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }
}
```

---

## 🔄 Integration with Existing Code

### Before (Current)

```rust
// In video-manager/src/handlers.rs
pub async fn serve_video(...) {
    // Scattered access logic
    if video.is_public {
        // serve
    } else if let Some(key) = access_key {
        // check key
    } else if let Some(user) = user_id {
        // check ownership or group
    }
}
```

### After (Refactored)

```rust
// In video-manager/src/handlers.rs
pub async fn serve_video(
    State(state): State<AppState>,
    session: Session,
    Query(params): Query<AccessParams>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let video = state.video_service.get_by_slug(&slug).await?;
    
    // Build access context
    let context = AccessContext {
        user_id: session.get_user_id(),
        access_key: params.key,
        resource_type: ResourceType::Video,
        resource_id: video.id,
        ip_address: extract_ip_from_request(),
        user_agent: extract_user_agent(),
        referer: extract_referer(),
        timestamp: time::OffsetDateTime::now_utc(),
    };
    
    // Check access (single call!)
    let decision = state.access_control
        .check_access(context, Permission::Read)
        .await?;
    
    if !decision.granted {
        return Err(Error::Forbidden(decision.reason));
    }
    
    // Serve the video
    Ok(serve_video_stream(&video))
}
```

### Benefits

✅ **Cleaner:** One function call instead of nested if statements  
✅ **Auditable:** Every access logged automatically  
✅ **Testable:** Easy to mock `AccessControlService`  
✅ **Debuggable:** `AccessDecision` explains why access was granted/denied  
✅ **Secure:** No SQL injection, type-safe queries  

---

## 📊 Permission Checks in Action

### Example 1: Public Video (Read Permission)

```rust
// Context
let context = AccessContext {
    user_id: None,
    access_key: None,
    resource_type: ResourceType::Video,
    resource_id: 123,
    ...
};

// Check
let decision = service.check_access(context, Permission::Read).await?;

// Result
AccessDecision {
    granted: true,
    layer: AccessLayer::Public,
    permission_requested: Permission::Read,
    permission_granted: Some(Permission::Read),
    reason: "Resource is publicly accessible",
    ...
}
```

### Example 2: Private Video with Access Key (Download Permission)

```rust
// Context
let context = AccessContext {
    user_id: None,
    access_key: Some("preview-2024"),
    resource_type: ResourceType::Video,
    resource_id: 456,
    ...
};

// Check
let decision = service.check_access(context, Permission::Download).await?;

// Result
AccessDecision {
    granted: true,
    layer: AccessLayer::AccessKey,
    permission_requested: Permission::Download,
    permission_granted: Some(Permission::Download),
    reason: "Access granted via key: Client Preview Q1",
    ...
}
```

### Example 3: Group Member (Edit Permission)

```rust
// Context
let context = AccessContext {
    user_id: Some("user123"),
    access_key: None,
    resource_type: ResourceType::Video,
    resource_id: 789,
    ...
};

// Check
let decision = service.check_access(context, Permission::Edit).await?;

// Result
AccessDecision {
    granted: true,
    layer: AccessLayer::GroupMembership,
    permission_requested: Permission::Edit,
    permission_granted: Some(Permission::Edit),
    reason: "Access granted via group membership (role: Editor)",
    ...
}
```

### Example 4: Access Denied

```rust
// Context
let context = AccessContext {
    user_id: None,
    access_key: None,
    resource_type: ResourceType::Video,
    resource_id: 999,
    ...
};

// Check
let decision = service.check_access(context, Permission::Read).await?;

// Result
AccessDecision {
    granted: false,
    layer: AccessLayer::Public,
    permission_requested: Permission::Read,
    permission_granted: None,
    reason: "No access layer granted permission",
    ...
}
```

---

## 🧪 Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_public_resource_grants_read() {
        let pool = setup_test_db().await;
        let service = AccessControlService::new(pool.clone());
        
        // Create public video
        create_test_video(&pool, 1, true, None).await;
        
        let context = AccessContext {
            user_id: None,
            access_key: None,
            resource_type: ResourceType::Video,
            resource_id: 1,
            ..Default::default()
        };
        
        let decision = service.check_access(context, Permission::Read).await.unwrap();
        
        assert!(decision.granted);
        assert_eq!(decision.layer, AccessLayer::Public);
    }
    
    #[tokio::test]
    async fn test_public_resource_denies_download() {
        let pool = setup_test_db().await;
        let service = AccessControlService::new(pool.clone());
        
        create_test_video(&pool, 1, true, None).await;
        
        let context = AccessContext {
            user_id: None,
            access_key: None,
            resource_type: ResourceType::Video,
            resource_id: 1,
            ..Default::default()
        };
        
        let decision = service.check_access(context, Permission::Download).await.unwrap();
        
        assert!(!decision.granted);
        assert_eq!(decision.reason, "Resource is not public or requires higher permission");
    }
    
    #[tokio::test]
    async fn test_owner_has_admin_access() {
        let pool = setup_test_db().await;
        let service = AccessControlService::new(pool.clone());
        
        create_test_video(&pool, 1, false, Some("user123")).await;
        
        let context = AccessContext {
            user_id: Some("user123".to_string()),
            access_key: None,
            resource_type: ResourceType::Video,
            resource_id: 1,
            ..Default::default()
        };
        
        let decision = service.check_access(context, Permission::Admin).await.unwrap();
        
        assert!(decision.granted);
        assert_eq!(decision.layer, AccessLayer::Ownership);
        assert_eq!(decision.permission_granted, Some(Permission::Admin));
    }
    
    #[tokio::test]
    async fn test_group_member_access_by_role() {
        let pool = setup_test_db().await;
        let service = AccessControlService::new(pool.clone());
        
        // Create group, video in group, add user as editor
        let group_id = create_test_group(&pool, "test-group").await;
        create_test_video(&pool, 1, false, Some("owner123")).await;
        update_video_group(&pool, 1, group_id).await;
        add_group_member(&pool, group_id, "user456", GroupRole::Editor).await;
        
        let context = AccessContext {
            user_id: Some("user456".to_string()),
            access_key: None,
            resource_type: ResourceType::Video,
            resource_id: 1,
            ..Default::default()
        };
        
        // Editor should have Edit permission
        let decision = service.check_access(context.clone(), Permission::Edit).await.unwrap();
        assert!(decision.granted);
        
        // But not Admin permission
        let decision = service.check_access(context, Permission::Admin).await.unwrap();
        assert!(!decision.granted);
    }
    
    #[tokio::test]
    async fn test_expired_access_key() {
        let pool = setup_test_db().await;
        let service = AccessControlService::new(pool.clone());
        
        create_test_video(&pool, 1, false, Some("owner123")).await;
        let key = create_expired_access_key(&pool, 1).await;
        
        let context = AccessContext {
            user_id: None,
            access_key: Some(key),
            resource_type: ResourceType::Video,
            resource_id: 1,
            ..Default::default()
        };
        
        let decision = service.check_access(context, Permission::Read).await.unwrap();
        
        assert!(!decision.granted);
        assert_eq!(decision.reason, "Access key has expired");
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    // Test real HTTP requests with access control
    
    #[tokio::test]
    async fn test_video_serve_with_access_key() {
        let app = create_test_app().await;
        
        let response = app
            .get("/watch/test-video?key=preview-2024")
            .send()
            .await;
        
        assert_eq!(response.status(), 200);
    }
    
    #[tokio::test]
    async fn test_video_serve_without_access() {
        let app = create_test_app().await;
        
        let response = app
            .get("/watch/private-video")
            .send()
            .await;
        
        assert_eq!(response.status(), 403);
    }
}
```

---

## 📁 File Structure

### New Crate: `crates/access-control/`

```
crates/access-control/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs                      # Public exports
│   ├── service.rs                  # AccessControlService
│   ├── models.rs                   # AccessDecision, AccessContext, AccessKeyData
│   ├── permissions.rs              # Permission enum and logic
│   ├── layers/                     # 4-layer implementation
│   │   ├── mod.rs
│   │   ├── public.rs               # Layer 1
│   │   ├── access_key.rs           # Layer 2
│   │   ├── group.rs                # Layer 3
│   │   └── owner.rs                # Layer 4
│   ├── repository.rs               # Type-safe DB queries
│   ├── audit.rs                    # Audit logging
│   └── error.rs                    # Access-specific errors
└── tests/
    ├── unit_tests.rs
    └── integration_tests.rs
```

### Updated `crates/common/`

```
crates/common/src/
├── lib.rs                          # Re-export access-control
├── types.rs                        # Keep ResourceType, GroupRole
├── error.rs                        # Keep common errors
├── db.rs                           # Keep DB utils
└── (remove access_control.rs)      # MOVED TO NEW CRATE
```

---

## 🚀 Implementation Plan

### Step 1: Create New Crate (Day 1)

```bash
# Create new crate
cargo new --lib crates/access-control

# Update workspace Cargo.toml
# Add to [workspace.members]
```

**Files to create:**
- `Cargo.toml` with dependencies
- `src/lib.rs` with module structure
- `README.md` with crate documentation

### Step 2: Define Core Models (Day 1-2)

**Create:**
- `src/models.rs` - AccessDecision, AccessContext, AccessKeyData
- `src/permissions.rs` - Permission enum with hierarchy
- `src/error.rs` - Access-specific errors

**Tasks:**
- Define all structs and enums
- Implement Display, Debug, etc.
- Add documentation
- Write unit tests for Permission logic

### Step 3: Build Repository Layer (Day 2-3)

**Create:**
- `src/repository.rs` - Type-safe database queries

**Tasks:**
- Implement all query methods
- Remove string concatenation
- Add proper error handling
- Write repository tests

### Step 4: Implement 4 Layers (Day 3-4)

**Create:**
- `src/layers/public.rs` - Layer 1
- `src/layers/access_key.rs` - Layer 2
- `src/layers/group.rs` - Layer 3
- `src/layers/owner.rs` - Layer 4

**Tasks:**
- Extract each layer into separate module
- Clean up logic
- Add comprehensive tests
- Document each layer

### Step 5: Build Main Service (Day 4-5)

**Create:**
- `src/service.rs` - AccessControlService

**Tasks:**
- Orchestrate all layers
- Implement priority-based decision
- Add convenience methods
- Integration tests

### Step 6: Add Audit Logging (Day 5)

**Create:**
- `src/audit.rs` - AuditLogger
- Migration script for audit table

**Tasks:**
- Implement logging
- Add query methods
- Performance optimization
- Privacy considerations

### Step 7: Update Dependents (Day 6-7)

**Update:**
- `crates/video-manager/` - Use new service
- `crates/image-manager/` - Use new service
- `crates/access-groups/` - Use new service
- `crates/access-codes/` - Use new service
- `src/main.rs` - Initialize new service

**Tasks:**
- Replace old access checks
- Update handler signatures
- Test all endpoints
- Update documentation

### Step 8: Database Migration (Day 7)

**Create:**
- `docs/migrations/004_access_control_refactor.sql`

**Tasks:**
- Create audit log table
- Add permission_level to access_keys
- Migrate existing data
- Test rollback

### Step 9: Testing & Documentation (Day 8)

**Tasks:**
- Run full test suite
- Update API documentation
- Create migration guide
- Performance benchmarks
- Security review

---

## 📝 Cargo.toml for New Crate

```toml
[package]
name = "access-control"
version = "0.1.0"
edition = "2021"

[dependencies]
# Database
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "sqlite"] }

# Async
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Time
time = { version = "0.3", features = ["parsing", "formatting"] }

# Logging
tracing = "0.1"

# Internal dependencies
common = { path = "../common" }

[dev-dependencies]
tokio-test = "0.4"
```

---

## 🔄 Migration Strategy

### Phase 1: Build New System (No Breaking Changes)

1. Create new `access-control` crate
2. Build all new code
3. **Don't touch existing code yet**
4. Test new system in isolation
5. Run benchmarks

### Phase 2: Parallel Implementation (Both Systems Running)

1. Add new service to AppState
2. Update ONE handler (e.g., video player) to use new system
3. Test thoroughly
4. Compare behavior with old system
5. Fix any discrepancies

### Phase 3: Gradual Migration

1. Update video-manager handlers (one by one)
2. Update image-manager handlers
3. Update access-groups handlers
4. Update access-codes handlers
5. Test after each migration

### Phase 4: Cleanup

1. Remove old `common/src/access_control.rs`
2. Remove old function calls
3. Update all documentation
4. Archive old code for reference

### Rollback Plan

If issues arise:
1. Keep old code in place until migration complete
2. Can switch handlers back to old system
3. Feature flag to toggle new/old system
4. Database changes are additive only (no data loss)

---

## 🎯 Success Metrics

### Performance

- [ ] Access checks < 50ms (99th percentile)
- [ ] No N+1 query problems
- [ ] Audit logging doesn't slow down requests
- [ ] Batch operations supported

### Security

- [ ] No SQL injection vulnerabilities
- [ ] All access decisions logged
- [ ] Failed access attempts tracked
- [ ] Security audit passes

### Code Quality

- [ ] 80%+ test coverage
- [ ] All functions documented
- [ ] No clippy warnings
- [ ] Consistent error handling

### User Experience

- [ ] Clear error messages (not just "Access Denied")
- [ ] Same behavior as old system
- [ ] No breaking changes to API
- [ ] Better debugging information

---

## 🔐 Security Considerations

### SQL Injection Prevention

```rust
// ❌ OLD (Vulnerable)
let query = format!("SELECT * FROM {} WHERE id = ?", table);

// ✅ NEW (Safe)
match resource_type {
    ResourceType::Video => {
        sqlx::query_scalar("SELECT * FROM videos WHERE id = ?")
            .bind(resource_id)
            .fetch_optional(&self.pool)
            .await?
    }
    // Explicit match for each type
}
```

### Audit Log Privacy

**What to log:**
- ✅ Access decisions (granted/denied)
- ✅ Permission levels
- ✅ Resource type and ID (not content)
- ✅ IP address (for security)
- ✅ Timestamp

**What NOT to log:**
- ❌ Sensitive resource content
- ❌ Full user details (just ID)
- ❌ Access key values (just existence)
- ❌ Personal information

### Rate Limiting (Future Enhancement)

```rust
// Check for suspicious activity
pub async fn check_rate_limit(
    &self,
    ip_address: &str,
    window_minutes: i32,
) -> Result<bool, Error> {
    let count: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM access_audit_log
         WHERE ip_address = ? 
           AND access_granted = 0
           AND created_at > datetime('now', '-' || ? || ' minutes')"
    )
    .bind(ip_address)
    .bind(window_minutes)
    .fetch_one(&self.pool)
    .await?;
    
    // More than 10 failed attempts in 5 minutes = suspicious
    Ok(count < 10)
}
```

---

## 📈 Performance Optimization

### Query Optimization

```rust
// Batch access checks for multiple resources
pub async fn check_batch_access(
    &self,
    context: AccessContext,
    resources: Vec<(ResourceType, i32)>,
    permission: Permission,
) -> Result<Vec<(i32, AccessDecision)>, Error> {
    // Single query to check all resources
    // Much faster than N individual checks
}
```

### Caching Strategy

```rust
// Cache public resources (they rarely change)
pub struct AccessControlService {
    pool: SqlitePool,
    repository: AccessRepository,
    audit_logger: AuditLogger,
    public_cache: Arc<RwLock<HashSet<(ResourceType, i32)>>>,
}

// Cache invalidation on visibility change
pub async fn invalidate_public_cache(&self, resource_type: ResourceType, resource_id: i32) {
    let mut cache = self.public_cache.write().await;
    cache.remove(&(resource_type, resource_id));
}
```

### Database Indexes

```sql
-- Ensure optimal query performance
CREATE INDEX IF NOT EXISTS idx_videos_public ON videos(is_public) WHERE is_public = 1;
CREATE INDEX IF NOT EXISTS idx_images_public ON images(is_public) WHERE is_public = 1;
CREATE INDEX IF NOT EXISTS idx_videos_group_id ON videos(group_id) WHERE group_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_images_group_id ON images(group_id) WHERE group_id IS NOT NULL;
```

---

## 🔄 Backwards Compatibility

### API Compatibility

**No breaking changes:**
- All existing endpoints work the same
- Same query parameters (`?key=...`)
- Same response formats
- Same error codes

**Enhanced features:**
- Better error messages
- Audit logging (invisible to users)
- Faster performance
- More secure

### Database Compatibility

**Additive only:**
- New table: `access_audit_log`
- New column: `access_keys.permission_level`
- No columns removed
- No data deleted
- Can rollback if needed

---

## 📋 Implementation Checklist

### Week 1: Foundation

- [ ] Create `crates/access-control/` directory
- [ ] Setup Cargo.toml with dependencies
- [ ] Create module structure
- [ ] Define Permission enum
- [ ] Define AccessDecision model
- [ ] Define AccessContext model
- [ ] Write permission hierarchy tests

### Week 2: Repository & Layers

- [ ] Create AccessRepository
- [ ] Implement type-safe queries
- [ ] Build Layer 1 (public)
- [ ] Build Layer 2 (access key)
- [ ] Build Layer 3 (group)
- [ ] Build Layer 4 (owner)
- [ ] Test each layer independently

### Week 3: Service & Audit

- [ ] Create AccessControlService
- [ ] Implement main check_access method
- [ ] Add layer prioritization
- [ ] Create AuditLogger
- [ ] Create audit log table
- [ ] Test service integration
- [ ] Performance benchmarks

### Week 4: Migration & Integration

- [ ] Add migration script
- [ ] Update video-manager
- [ ] Update image-manager
- [ ] Update access-groups
- [ ] Update access-codes
- [ ] Full integration tests
- [ ] Update documentation
- [ ] Deploy and monitor

---

## 🎓 Key Design Decisions

### Decision 1: Separate Crate vs Module?

**Choice:** ✅ Separate Crate (`crates/access-control/`)

**Reasoning:**
- Access control is a core concern, deserves own crate
- Can be tested independently
- Clear boundaries and ownership
- Reusable in other projects
- Follows existing pattern (access-groups, access-codes, etc.)

### Decision 2: Trait-Based vs Direct Service?

**Choice:** ✅ Trait-Based with Concrete Implementation

**Reasoning:**
- Mockable for testing
- Can swap implementations (e.g., Redis-based for scale)
- Follows Rust best practices
- Type-safe polymorphism

### Decision 3: Audit All or Critical Only?

**Choice:** ✅ Audit All Access Decisions

**Reasoning:**
- Compliance requirements
- Security monitoring
- Debugging support
- Usage analytics
- Minimal performance impact with proper indexing

### Decision 4: Synchronous or Async?

**Choice:** ✅ Async (Required)

**Reasoning:**
- Database calls are async
- HTTP handlers are async
- Non-blocking IO
- Better scalability

### Decision 5: Permission Granularity?

**Choice:** ✅ 5 Levels (Read, Download, Edit, Delete, Admin)

**Reasoning:**
- Read: Public content, previews
- Download: Access keys for distribution
- Edit: Group collaborators
- Delete: Senior team members
- Admin: Owners, group admins

---

## 🧩 Integration Examples

### Example 1: Video Handler (Before & After)

**Before:**
```rust
pub async fn serve_video(
    State(state): State<AppState>,
    session: Session,
    Query(params): Query<VideoParams>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let video = state.video_service.get_by_slug(&slug).await?;
    
    // OLD: Manual access check (scattered logic)
    let user_id = session.get::<String>("user_id").ok();
    let has_access = check_resource_access(
        &state.pool,
        user_id.as_deref(),
        params.key.as_deref(),
        ResourceType::Video,
        video.id,
    ).await?;
    
    if !has_access {
        return Err(Error::Forbidden);
    }
    
    serve_video_file(&video)
}
```

**After:**
```rust
pub async fn serve_video(
    State(state): State<AppState>,
    session: Session,
    Query(params): Query<VideoParams>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let video = state.video_service.get_by_slug(&slug).await?;
    
    // NEW: Clean, type-safe access check
    let context = AccessContext {
        user_id: session.get("user_id").ok(),
        access_key: params.key,
        resource_type: ResourceType::Video,
        resource_id: video.id,
        ip_address: extract_ip(&req),
        user_agent: extract_user_agent(&req),
        referer: extract_referer(&req),
        timestamp: time::OffsetDateTime::now_utc(),
    };
    
    let decision = state.access_control
        .check_access(context, Permission::Read)
        .await?;
    
    if !decision.granted {
        return Err(Error::Forbidden(decision.reason));
    }
    
    serve_video_file(&video)
}
```

### Example 2: Download Handler (New Permission Level)

```rust
pub async fn download_video(
    State(state): State<AppState>,
    session: Session,
    Query(params): Query<VideoParams>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let video = state.video_service.get_by_slug(&slug).await?;
    
    let context = AccessContext {
        user_id: session.get("user_id").ok(),
        access_key: params.key,
        resource_type: ResourceType::Video,
        resource_id: video.id,
        ..Default::default()
    };
    
    // NEW: Check Download permission (not just Read)
    let decision = state.access_control
        .check_access(context, Permission::Download)
        .await?;
    
    if !decision.granted {
        return Err(Error::Forbidden(format!(
            "Download not allowed: {}",
            decision.reason
        )));
    }
    
    // Track download
    if let Some(key) = &context.access_key {
        state.access_control.increment_download_count(key).await?;
    }
    
    serve_video_download(&video)
}
```

### Example 3: Edit Handler (Permission Check)

```rust
pub async fn update_video(
    State(state): State<AppState>,
    session: Session,
    Path(slug): Path<String>,
    Json(data): Json<UpdateVideoData>,
) -> Result<impl IntoResponse, Error> {
    let video = state.video_service.get_by_slug(&slug).await?;
    
    let context = AccessContext {
        user_id: session.get("user_id").ok(),
        access_key: None,
        resource_type: ResourceType::Video,
        resource_id: video.id,
        ..Default::default()
    };
    
    // NEW: Check Edit permission
    let decision = state.access_control
        .check_access(context, Permission::Edit)
        .await?;
    
    if !decision.granted {
        return Err(Error::Forbidden(decision.reason));
    }
    
    // Update video
    state.video_service.update(video.id, data).await?;
    
    Ok(Json(json!({ "success": true })))
}
```

---

## 🧪 Test Scenarios

### Scenario 1: Public Resource

| Context | Permission | Expected Result |
|---------|-----------|-----------------|
| Anonymous | Read | ✅ Granted (Layer 1) |
| Anonymous | Download | ❌ Denied |
| User | Read | ✅ Granted (Layer 1) |
| User | Edit | ❌ Denied |
| Owner | Admin | ✅ Granted (Layer 4) |

### Scenario 2: Private Resource with Access Key

| Context | Permission | Expected Result |
|---------|-----------|-----------------|
| Anonymous + no key | Read | ❌ Denied |
| Anonymous + valid key | Read | ✅ Granted (Layer 2) |
| Anonymous + expired key | Read | ❌ Denied (expired) |
| Anonymous + limited key | Download | ✅ Granted (if under limit) |

### Scenario 3: Group Resource

| Context | Permission | Expected Result |
|---------|-----------|-----------------|
| Non-member | Read | ❌ Denied |
| Viewer | Read | ✅ Granted (Layer 3) |
| Viewer | Edit | ❌ Denied (insufficient role) |
| Editor | Edit | ✅ Granted (Layer 3) |
| Admin | Delete | ✅ Granted (Layer 3) |
| Owner | Admin | ✅ Granted (Layer 4) |

### Scenario 4: Mixed Access (Multiple Layers)

```
Resource: Private video in group
User: Group member (Viewer role)
Access Key: Also provided (grants Download)

Layer 1 (Public): Denied (not public)
Layer 2 (Access Key): Granted (Download)
Layer 3 (Group): Granted (Read via Viewer role)
Layer 4 (Ownership): Denied (not owner)

Result: Access Key wins (higher permission)
Permission Granted: Download
```

---

## 📊 Comparison: Old vs New

| Aspect | Old System | New System |
|--------|-----------|------------|
| **SQL Safety** | ❌ String concat | ✅ Type-safe |
| **Permissions** | ❌ Boolean only | ✅ 5 granular levels |
| **Audit Trail** | ⚠️ Partial (keys only) | ✅ Complete |
| **Testability** | ❌ Hard to mock | ✅ Easy to test |
| **Error Messages** | ❌ Generic | ✅ Descriptive |
| **Code Organization** | ❌ Scattered | ✅ Centralized |
| **Trait Usage** | ❌ Unused | ✅ Implemented |
| **Layer Model** | ⚠️ Unclear | ✅ Well-defined |
| **Performance** | ⚠️ Multiple queries | ✅ Optimized |
| **Extensibility** | ❌ Hard to extend | ✅ Easy to add features |

---

## 🎯 Next Steps

1. **Review this design document** with team
2. **Get approval** for architectural changes
3. **Create implementation tasks** (GitHub issues or similar)
4. **Start coding** Step 1 (create new crate)
5. **Iterate** based on feedback

---

## 📚 References

### Internal Docs
- `MASTER_PLAN.md` (Lines 477-595) - Access Control Models
- `GROUP_OWNERSHIP_EXPLAINED.md` - Group permissions
- `ACCESS_CODE_DECISION_GUIDE.md` - Access code patterns

### External Resources
- [OWASP Access Control](https://owasp.org/www-community/Access_Control)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [SQLx Documentation](https://docs.rs/sqlx/)

---

## ✅ Approval Checklist

- [ ] Architecture reviewed
- [ ] Security implications understood
- [ ] Performance acceptable
- [ ] Migration path clear
- [ ] Rollback plan defined
- [ ] Testing strategy approved
- [ ] Timeline realistic (4 weeks)
- [ ] Ready to implement

---

**Document Status:** 📋 Ready for Review  
**Estimated Implementation:** 4 weeks  
**Risk Level:** Medium (internal refactor, no API changes)  
**Breaking Changes:** None (backwards compatible)

---

**Next Document:** Create implementation guide once approved  
**Implementation Start:** After approval  
**Target Completion:** 4 weeks from start

---

**Version:** 1.0  
**Created:** February 2026  
**Last Updated:** February 2026  
**Status:** 🎨 Design Phase