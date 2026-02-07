pub mod renderer;
#[cfg(test)]
mod tests;

use anyhow::Result;

use crate::models::*;
use crate::storage::Store;

/// Generate role-filtered, level-appropriate project context from stored memory.
///
/// Auto-starts a session if none is active.
pub fn generate_context(
    store: &Store,
    role: &ContextRole,
    level: &ContextLevel,
) -> Result<ContextOutput> {
    let meta = store.get_project_meta()?;
    let decisions = store.list_decisions(Some(&DecisionStatus::Active))?;
    let tasks = store.list_tasks(None)?;
    let files = store.list_file_summaries()?;
    let sessions = store.list_sessions(5)?;

    // Auto-start session if none active
    let active_session = match store.get_active_session()? {
        Some(s) => Some(s),
        None => {
            let goal = format!("{} context request", role.as_str());
            Some(store.start_session("auto", &goal, &[])?)
        }
    };

    let markdown = renderer::render(&renderer::RenderContext {
        role,
        level,
        meta: &meta,
        decisions: &decisions,
        tasks: &tasks,
        files: &files,
        sessions: &sessions,
        active_session: &active_session,
    });
    let estimated_tokens = estimate_tokens(&markdown);

    Ok(ContextOutput {
        role: role.clone(),
        level: level.clone(),
        markdown,
        estimated_tokens,
    })
}

/// Estimate token count using word-based heuristic.
/// English text with markdown averages ~1.3 tokens per word.
/// Markdown symbols and punctuation add overhead.
pub fn estimate_tokens(text: &str) -> usize {
    let words = text.split_whitespace().count();
    let non_ascii_chars = text.chars().filter(|c| !c.is_ascii()).count();
    // ~1.3 tokens per word + 1 token per non-ASCII char (emoji, etc.)
    ((words as f64 * 1.3) as usize) + non_ascii_chars
}
