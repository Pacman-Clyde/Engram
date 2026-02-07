use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetContextParams {
    /// Role filter: build, review, debug, resume
    #[schemars(
        description = "Context role: build (tasks + architecture), review (conventions + files), debug (recent changes + issues), resume (last session handoff)"
    )]
    pub role: Option<String>,
    /// Detail level: minimal (~200 tokens), standard (~1000 tokens), full (all details)
    #[schemars(description = "Detail level: minimal, standard, full. Defaults to standard.")]
    pub level: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddDecisionParams {
    /// Short title for the decision
    #[schemars(description = "Decision title, e.g. 'Use rusqlite over sqlx'")]
    pub title: String,
    /// Problem context / why the decision was needed
    #[schemars(description = "Context explaining why this decision was needed")]
    pub context: String,
    /// The decision that was made
    #[schemars(description = "What was decided")]
    pub decision: String,
    /// Alternatives that were considered
    #[schemars(description = "Alternatives considered (optional)")]
    pub alternatives: Option<Vec<String>>,
    /// Tags for categorization
    #[schemars(description = "Tags for categorization (optional)")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateTaskParams {
    /// Task ID (or prefix)
    #[schemars(description = "Task ID or ID prefix to match")]
    pub id: String,
    /// New status: todo, in_progress, done, blocked
    #[schemars(description = "New status: todo, in_progress, done, blocked")]
    pub status: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SummarizeFileParams {
    /// File path relative to project root
    #[schemars(description = "File path relative to project root")]
    pub path: String,
    /// What the file does, key types, purpose
    #[schemars(description = "Summary of the file's purpose and contents")]
    pub summary: String,
    /// Key types/structs/functions defined in the file
    #[schemars(description = "Key types, structs, or functions (optional)")]
    pub key_types: Option<Vec<String>>,
    /// Dependencies this file relies on
    #[schemars(description = "Dependencies or imports (optional)")]
    pub dependencies: Option<Vec<String>>,
    /// Tags for categorization
    #[schemars(description = "Tags for categorization (optional)")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StartSessionParams {
    /// Name of the agent starting the session
    #[schemars(description = "Agent name, e.g. 'claude', 'gpt', 'cursor'")]
    pub agent: Option<String>,
    /// What this session aims to accomplish
    #[schemars(description = "Goal for this work session")]
    pub goal: String,
    /// Tags for categorization
    #[schemars(description = "Tags for categorization (optional)")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EndSessionParams {
    /// Handoff notes for the next agent/session
    #[schemars(description = "Handoff notes: what was done, what's next, any blockers")]
    pub handoff: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchParams {
    /// Search query (FTS5 syntax)
    #[schemars(description = "Search query text")]
    pub query: String,
    /// Filter by entity type: decision, task, file, session
    #[schemars(description = "Filter by type: decision, task, file, session (optional)")]
    pub entity_type: Option<String>,
}
