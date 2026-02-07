use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Top-level project metadata: name, description, tech stack, and conventions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    pub description: String,
    pub stack: Vec<String>,
    pub conventions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
