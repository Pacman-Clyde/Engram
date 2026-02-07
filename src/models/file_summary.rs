use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Documentation of a source file's purpose, key types, and dependencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSummary {
    pub id: String,
    pub path: String,
    pub summary: String,
    pub key_types: Vec<String>,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
    pub content_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
