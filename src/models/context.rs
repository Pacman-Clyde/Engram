use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Role filter that controls which memory types are prioritized in context output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContextRole {
    /// Architecture and task focus.
    Build,
    /// Conventions and file summaries.
    Review,
    /// Recent changes and completed tasks.
    Debug,
    /// Last session handoff for continuity.
    Resume,
}

impl ContextRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Review => "review",
            Self::Debug => "debug",
            Self::Resume => "resume",
        }
    }
}

impl FromStr for ContextRole {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "build" => Ok(Self::Build),
            "review" => Ok(Self::Review),
            "debug" => Ok(Self::Debug),
            "resume" => Ok(Self::Resume),
            _ => anyhow::bail!("invalid context role: {s}"),
        }
    }
}

impl fmt::Display for ContextRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Detail level controlling how much information is included in context output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContextLevel {
    /// ~50-200 tokens: next task and last handoff only.
    Minimal,
    /// ~200-1000 tokens: decisions, active tasks, last session.
    Standard,
    /// All details: conventions, full task list, file summaries, session history.
    Full,
}

impl ContextLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Minimal => "minimal",
            Self::Standard => "standard",
            Self::Full => "full",
        }
    }
}

impl FromStr for ContextLevel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "minimal" => Ok(Self::Minimal),
            "standard" => Ok(Self::Standard),
            "full" => Ok(Self::Full),
            _ => anyhow::bail!("invalid context level: {s}"),
        }
    }
}

impl fmt::Display for ContextLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Result of context generation, containing the rendered markdown and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextOutput {
    pub role: ContextRole,
    pub level: ContextLevel,
    /// Rendered markdown context string.
    pub markdown: String,
    /// Estimated token count (word-based heuristic).
    pub estimated_tokens: usize,
}
