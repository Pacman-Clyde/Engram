//! Engram: AI agent memory system.
//!
//! Provides structured, token-efficient project context for AI agents.
//! Stores decisions, tasks, file summaries, and session handoffs in a local
//! SQLite database, accessible via CLI or MCP server.

/// CLI command definitions and handlers.
pub mod cli;
/// Context generation engine with role-based filtering and token estimation.
pub mod engine;
/// Model Context Protocol (MCP) server over stdio.
pub mod mcp;
/// Data models for all memory types.
pub mod models;
/// SQLite storage layer with CRUD operations and FTS5 search.
pub mod storage;

use std::path::PathBuf;

use anyhow::Result;

/// Locate the `.engram/memory.db` database file in the current directory.
///
/// Returns an error if no `.engram` directory exists.
pub fn find_engram_db() -> Result<PathBuf> {
    let dir = PathBuf::from(".engram");
    if !dir.exists() {
        anyhow::bail!("not an engram project (run `engram init` first)");
    }
    Ok(dir.join("memory.db"))
}
