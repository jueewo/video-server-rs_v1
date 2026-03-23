use serde::{Deserialize, Serialize};

/// Resource types supported by the system
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    Video,
    Image,
    File,
    Folder,
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceType::Video => write!(f, "video"),
            ResourceType::Image => write!(f, "image"),
            ResourceType::File => write!(f, "document"),
            ResourceType::Folder => write!(f, "folder"),
        }
    }
}

impl std::str::FromStr for ResourceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "video" => Ok(ResourceType::Video),
            "image" => Ok(ResourceType::Image),
            "file" | "document" => Ok(ResourceType::File),
            "folder" => Ok(ResourceType::Folder),
            _ => Err(format!("Invalid resource type: {}", s)),
        }
    }
}

/// Permission levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    Read,
    Write,
    Delete,
    Share,
    Admin,
}

/// Group member roles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GroupRole {
    Owner,
    Admin,
    Editor,
    Contributor,
    Viewer,
}

impl GroupRole {
    pub fn can_read(&self) -> bool {
        matches!(
            self,
            GroupRole::Owner
                | GroupRole::Admin
                | GroupRole::Editor
                | GroupRole::Contributor
                | GroupRole::Viewer
        )
    }

    pub fn can_write(&self) -> bool {
        matches!(
            self,
            GroupRole::Owner | GroupRole::Admin | GroupRole::Editor | GroupRole::Contributor
        )
    }

    pub fn can_delete(&self) -> bool {
        matches!(
            self,
            GroupRole::Owner | GroupRole::Admin | GroupRole::Editor
        )
    }

    pub fn can_admin(&self) -> bool {
        matches!(self, GroupRole::Owner | GroupRole::Admin)
    }
}

impl std::fmt::Display for GroupRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GroupRole::Owner => write!(f, "owner"),
            GroupRole::Admin => write!(f, "admin"),
            GroupRole::Editor => write!(f, "editor"),
            GroupRole::Contributor => write!(f, "contributor"),
            GroupRole::Viewer => write!(f, "viewer"),
        }
    }
}

impl std::str::FromStr for GroupRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "owner" => Ok(GroupRole::Owner),
            "admin" => Ok(GroupRole::Admin),
            "editor" => Ok(GroupRole::Editor),
            "contributor" => Ok(GroupRole::Contributor),
            "viewer" => Ok(GroupRole::Viewer),
            _ => Err(format!("Invalid group role: {}", s)),
        }
    }
}

/// Common resource metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetadata {
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub owner_id: String,
    pub group_id: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
    pub is_public: bool,
    pub resource_type: ResourceType,
}

