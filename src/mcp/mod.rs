pub mod tools;

use std::str::FromStr;
use std::sync::{Arc, Mutex};

use rmcp::{
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError, ServerHandler, ServiceExt,
};

use crate::engine;
use crate::models::{ContextLevel, ContextRole, TaskStatus as EngramTaskStatus};
use crate::storage::Store;
use tools::*;

#[derive(Clone)]
pub struct EngramServer {
    store: Arc<Mutex<Store>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl EngramServer {
    pub fn new(store: Store) -> Self {
        Self {
            store: Arc::new(Mutex::new(store)),
            tool_router: Self::tool_router(),
        }
    }

    fn with_store<F, T>(&self, f: F) -> Result<T, McpError>
    where
        F: FnOnce(&Store) -> anyhow::Result<T>,
    {
        let store = self
            .store
            .lock()
            .map_err(|e| McpError::internal_error(format!("lock poisoned: {e}"), None))?;
        f(&store).map_err(|e| McpError::internal_error(format!("{e:#}"), None))
    }

    #[tool(
        description = "Get token-efficient project context. Returns role-filtered, level-appropriate markdown summary of the project state including decisions, tasks, files, and session history."
    )]
    fn get_context(
        &self,
        Parameters(params): Parameters<GetContextParams>,
    ) -> Result<CallToolResult, McpError> {
        let role = params.role.as_deref().unwrap_or("build");
        let level = params.level.as_deref().unwrap_or("standard");

        let role = ContextRole::from_str(role)
            .map_err(|e| McpError::invalid_params(format!("{e}"), None))?;
        let level = ContextLevel::from_str(level)
            .map_err(|e| McpError::invalid_params(format!("{e}"), None))?;

        let output = self.with_store(|store| engine::generate_context(store, &role, &level))?;

        Ok(CallToolResult::success(vec![Content::text(
            output.markdown,
        )]))
    }

    #[tool(
        description = "Record an architectural decision with context, rationale, and alternatives considered."
    )]
    fn add_decision(
        &self,
        Parameters(params): Parameters<AddDecisionParams>,
    ) -> Result<CallToolResult, McpError> {
        let alts = params.alternatives.unwrap_or_default();
        let tags = params.tags.unwrap_or_default();

        let decision = self.with_store(|store| {
            store.add_decision(
                &params.title,
                &params.context,
                &params.decision,
                &alts,
                &tags,
            )
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Decision recorded: {} ({})",
            decision.title,
            &decision.id[..8.min(decision.id.len())]
        ))]))
    }

    #[tool(description = "Update a task's status. Use ID prefix matching.")]
    fn update_task(
        &self,
        Parameters(params): Parameters<UpdateTaskParams>,
    ) -> Result<CallToolResult, McpError> {
        let status = EngramTaskStatus::from_str(&params.status)
            .map_err(|e| McpError::invalid_params(format!("{e}"), None))?;

        self.with_store(|store| {
            let tasks = store.list_tasks(None)?;
            let found = tasks
                .iter()
                .find(|t| t.id.starts_with(&params.id))
                .ok_or_else(|| anyhow::anyhow!("task not found with ID prefix: {}", params.id))?;
            store.update_task_status(&found.id, &status)?;
            Ok(format!(
                "Task '{}' ({}) updated to {}",
                found.title,
                &found.id[..8.min(found.id.len())],
                status.as_str()
            ))
        })
        .map(|msg| CallToolResult::success(vec![Content::text(msg)]))
    }

    #[tool(
        description = "Add or update a file summary. Tracks what each file does, key types, and dependencies."
    )]
    fn summarize_file(
        &self,
        Parameters(params): Parameters<SummarizeFileParams>,
    ) -> Result<CallToolResult, McpError> {
        let key_types = params.key_types.unwrap_or_default();
        let deps = params.dependencies.unwrap_or_default();
        let tags = params.tags.unwrap_or_default();

        // Compute content hash if file exists
        let content_hash = if std::path::Path::new(&params.path).exists() {
            use sha2::{Digest, Sha256};
            let content = std::fs::read(&params.path)
                .map_err(|e| McpError::internal_error(format!("read file: {e}"), None))?;
            format!("{:x}", Sha256::digest(&content))
        } else {
            "unknown".to_string()
        };

        let file = self.with_store(|store| {
            store.upsert_file_summary(
                &params.path,
                &params.summary,
                &key_types,
                &deps,
                &tags,
                &content_hash,
            )
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "File summary saved: {}",
            file.path
        ))]))
    }

    #[tool(description = "Start a new work session. Only one session can be active at a time.")]
    fn start_session(
        &self,
        Parameters(params): Parameters<StartSessionParams>,
    ) -> Result<CallToolResult, McpError> {
        let agent = params.agent.as_deref().unwrap_or("unknown");
        let tags = params.tags.unwrap_or_default();

        let session = self.with_store(|store| {
            if let Some(active) = store.get_active_session()? {
                anyhow::bail!(
                    "session already active: {} ({}) - end it first",
                    &active.id[..8.min(active.id.len())],
                    active.goal
                );
            }
            store.start_session(agent, &params.goal, &tags)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Session started: {} (agent: {}, goal: {})",
            &session.id[..8.min(session.id.len())],
            session.agent,
            session.goal
        ))]))
    }

    #[tool(description = "End the active session with handoff notes for the next agent.")]
    fn end_session(
        &self,
        Parameters(params): Parameters<EndSessionParams>,
    ) -> Result<CallToolResult, McpError> {
        let msg = self.with_store(|store| {
            let active = store
                .get_active_session()?
                .ok_or_else(|| anyhow::anyhow!("no active session to end"))?;
            store.end_session(&active.id, &params.handoff)?;
            Ok(format!(
                "Session ended: {} - handoff recorded",
                &active.id[..8.min(active.id.len())]
            ))
        })?;

        Ok(CallToolResult::success(vec![Content::text(msg)]))
    }

    #[tool(
        description = "Full-text search across all memory: decisions, tasks, file summaries, sessions."
    )]
    fn search(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let results = self.with_store(|store| {
            let hits = store.search(&params.query, params.entity_type.as_deref())?;
            let mut lines = Vec::new();
            for (id, etype) in &hits {
                let short_id = &id[..8.min(id.len())];
                let line = match etype.as_str() {
                    "decision" => {
                        if let Some(d) = store.get_decision(id)? {
                            format!("[decision:{}] {} - {}", short_id, d.title, d.decision)
                        } else {
                            format!("[decision:{}]", short_id)
                        }
                    }
                    "task" => {
                        if let Some(t) = store.get_task(id)? {
                            format!("[task:{}] ({}) {}", short_id, t.status.as_str(), t.title)
                        } else {
                            format!("[task:{}]", short_id)
                        }
                    }
                    "file" => {
                        if let Some(f) = store.get_file_summary(id)? {
                            format!("[file:{}] {} - {}", short_id, f.path, f.summary)
                        } else {
                            format!("[file:{}]", short_id)
                        }
                    }
                    "session" => {
                        if let Some(s) = store.get_session(id)? {
                            let status = if s.ended_at.is_some() {
                                "ended"
                            } else {
                                "active"
                            };
                            format!(
                                "[session:{}] ({}) {} - {}",
                                short_id, status, s.agent, s.goal
                            )
                        } else {
                            format!("[session:{}]", short_id)
                        }
                    }
                    _ => format!("[{}:{}]", etype, short_id),
                };
                lines.push(line);
            }
            if lines.is_empty() {
                Ok(format!("No results for: {}", params.query))
            } else {
                Ok(format!("{}\n\n{} result(s)", lines.join("\n"), lines.len()))
            }
        })?;

        Ok(CallToolResult::success(vec![Content::text(results)]))
    }
}

#[tool_handler]
impl ServerHandler for EngramServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "engram".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                title: None,
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Engram: AI agent memory system. Use get_context to orient yourself in the project. \
                 Record decisions, update tasks, summarize files, and manage sessions as you work."
                    .into(),
            ),
        }
    }
}

pub async fn serve() -> anyhow::Result<()> {
    let db_path = crate::find_engram_db()?;
    let store = Store::open(&db_path)?;
    let server = EngramServer::new(store);

    eprintln!("engram: MCP server starting on stdio");
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
