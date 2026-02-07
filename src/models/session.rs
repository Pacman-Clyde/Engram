use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A work session tracking which agent worked on what, with handoff notes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub agent: String,
    pub goal: String,
    pub handoff: Option<String>,
    pub tags: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}
