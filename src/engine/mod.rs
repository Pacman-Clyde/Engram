pub mod renderer;
#[cfg(test)]
mod tests;

use anyhow::Result;

use crate::models::*;
use crate::storage::Store;

pub fn generate_context(store: &Store, role: &ContextRole, level: &ContextLevel) -> Result<ContextOutput> {
    let meta = store.get_project_meta()?;
    let decisions = store.list_decisions(Some(&DecisionStatus::Active))?;
    let tasks = store.list_tasks(None)?;
    let files = store.list_file_summaries()?;
    let sessions = store.list_sessions(5)?;
    let active_session = store.get_active_session()?;

    let markdown = renderer::render(role, level, &meta, &decisions, &tasks, &files, &sessions, &active_session);
    let estimated_tokens = markdown.len() / 4; // rough estimate: 4 chars per token

    Ok(ContextOutput {
        role: role.clone(),
        level: level.clone(),
        markdown,
        estimated_tokens,
    })
}
