use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContextRole {
    Build,
    Review,
    Debug,
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

    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "build" => Ok(Self::Build),
            "review" => Ok(Self::Review),
            "debug" => Ok(Self::Debug),
            "resume" => Ok(Self::Resume),
            _ => anyhow::bail!("invalid context role: {s}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContextLevel {
    Minimal,
    Standard,
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

    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "minimal" => Ok(Self::Minimal),
            "standard" => Ok(Self::Standard),
            "full" => Ok(Self::Full),
            _ => anyhow::bail!("invalid context level: {s}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextOutput {
    pub role: ContextRole,
    pub level: ContextLevel,
    pub markdown: String,
    pub estimated_tokens: usize,
}
