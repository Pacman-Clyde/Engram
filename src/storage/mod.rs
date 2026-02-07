pub mod schema;

use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::models::*;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path).context("failed to open database")?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .context("failed to set pragmas")?;
        conn.execute_batch(schema::CREATE_TABLES).context("failed to create schema")?;
        Ok(Self { conn })
    }

    pub fn open_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("failed to open in-memory database")?;
        conn.execute_batch(schema::CREATE_TABLES).context("failed to create schema")?;
        Ok(Self { conn })
    }

    // ── Project Meta ──

    pub fn get_project_meta(&self) -> Result<Option<ProjectMeta>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, description, stack, conventions, created_at, updated_at FROM project_meta LIMIT 1",
        )?;
        let mut rows = stmt.query_map([], |row| {
            Ok(ProjectMeta {
                name: row.get(0)?,
                description: row.get(1)?,
                stack: serde_json::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                conventions: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
                created_at: row.get::<_, String>(4)?.parse().unwrap(),
                updated_at: row.get::<_, String>(5)?.parse().unwrap(),
            })
        })?;
        match rows.next() {
            Some(Ok(meta)) => Ok(Some(meta)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn set_project_meta(&self, name: &str, description: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        // Upsert: delete old row then insert
        self.conn.execute("DELETE FROM project_meta", [])?;
        self.conn.execute(
            "INSERT INTO project_meta (name, description, stack, conventions, created_at, updated_at) VALUES (?1, ?2, '[]', '[]', ?3, ?3)",
            params![name, description, now],
        )?;
        Ok(())
    }

    pub fn update_project_meta_stack(&self, stack: &[String]) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let stack_json = serde_json::to_string(stack)?;
        self.conn.execute(
            "UPDATE project_meta SET stack = ?1, updated_at = ?2",
            params![stack_json, now],
        )?;
        Ok(())
    }

    pub fn update_project_meta_conventions(&self, conventions: &[String]) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let conv_json = serde_json::to_string(conventions)?;
        self.conn.execute(
            "UPDATE project_meta SET conventions = ?1, updated_at = ?2",
            params![conv_json, now],
        )?;
        Ok(())
    }

    // ── Decisions ──

    pub fn add_decision(
        &self,
        title: &str,
        context: &str,
        decision: &str,
        alternatives: &[String],
        tags: &[String],
    ) -> Result<Decision> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        let alts_json = serde_json::to_string(alternatives)?;
        let tags_json = serde_json::to_string(tags)?;

        self.conn.execute(
            "INSERT INTO decisions (id, title, context, decision, alternatives, tags, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'active', ?7, ?7)",
            params![id, title, context, decision, alts_json, tags_json, now_str],
        )?;

        // Index for FTS5
        let search_content = format!("{title} {context} {decision}");
        self.conn.execute(
            "INSERT INTO search_index (entity_id, entity_type, body) VALUES (?1, 'decision', ?2)",
            params![id, search_content],
        )?;

        Ok(Decision {
            id,
            title: title.to_string(),
            context: context.to_string(),
            decision: decision.to_string(),
            alternatives: alternatives.to_vec(),
            tags: tags.to_vec(),
            status: DecisionStatus::Active,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn list_decisions(&self, status: Option<&DecisionStatus>) -> Result<Vec<Decision>> {
        let sql = match status {
            Some(_) => "SELECT id, title, context, decision, alternatives, tags, status, created_at, updated_at FROM decisions WHERE status = ?1 ORDER BY created_at DESC",
            None => "SELECT id, title, context, decision, alternatives, tags, status, created_at, updated_at FROM decisions ORDER BY created_at DESC",
        };
        let mut stmt = self.conn.prepare(sql)?;
        let rows = if let Some(s) = status {
            stmt.query_map(params![s.as_str()], Self::map_decision)?
        } else {
            stmt.query_map([], Self::map_decision)?
        };
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    pub fn get_decision(&self, id: &str) -> Result<Option<Decision>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, context, decision, alternatives, tags, status, created_at, updated_at FROM decisions WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], Self::map_decision)?;
        match rows.next() {
            Some(Ok(d)) => Ok(Some(d)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    fn map_decision(row: &rusqlite::Row<'_>) -> rusqlite::Result<Decision> {
        Ok(Decision {
            id: row.get(0)?,
            title: row.get(1)?,
            context: row.get(2)?,
            decision: row.get(3)?,
            alternatives: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
            tags: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
            status: DecisionStatus::from_str(&row.get::<_, String>(6)?).unwrap(),
            created_at: row.get::<_, String>(7)?.parse().unwrap(),
            updated_at: row.get::<_, String>(8)?.parse().unwrap(),
        })
    }

    // ── Tasks ──

    pub fn add_task(
        &self,
        title: &str,
        description: &str,
        priority: &TaskPriority,
        phase: Option<&str>,
        tags: &[String],
    ) -> Result<Task> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        let tags_json = serde_json::to_string(tags)?;

        self.conn.execute(
            "INSERT INTO tasks (id, title, description, status, priority, phase, tags, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'todo', ?4, ?5, ?6, ?7, ?7)",
            params![id, title, description, priority.as_str(), phase, tags_json, now_str],
        )?;

        let search_content = format!("{title} {description}");
        self.conn.execute(
            "INSERT INTO search_index (entity_id, entity_type, body) VALUES (?1, 'task', ?2)",
            params![id, search_content],
        )?;

        Ok(Task {
            id,
            title: title.to_string(),
            description: description.to_string(),
            status: TaskStatus::Todo,
            priority: priority.clone(),
            phase: phase.map(String::from),
            tags: tags.to_vec(),
            created_at: now,
            updated_at: now,
        })
    }

    pub fn update_task_status(&self, id: &str, status: &TaskStatus) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let affected = self.conn.execute(
            "UPDATE tasks SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.as_str(), now, id],
        )?;
        if affected == 0 {
            anyhow::bail!("task not found: {id}");
        }
        Ok(())
    }

    pub fn list_tasks(&self, status: Option<&TaskStatus>) -> Result<Vec<Task>> {
        let sql = match status {
            Some(_) => "SELECT id, title, description, status, priority, phase, tags, created_at, updated_at FROM tasks WHERE status = ?1 ORDER BY created_at DESC",
            None => "SELECT id, title, description, status, priority, phase, tags, created_at, updated_at FROM tasks ORDER BY created_at DESC",
        };
        let mut stmt = self.conn.prepare(sql)?;
        let rows = if let Some(s) = status {
            stmt.query_map(params![s.as_str()], Self::map_task)?
        } else {
            stmt.query_map([], Self::map_task)?
        };
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    pub fn get_task(&self, id: &str) -> Result<Option<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, status, priority, phase, tags, created_at, updated_at FROM tasks WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], Self::map_task)?;
        match rows.next() {
            Some(Ok(t)) => Ok(Some(t)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    fn map_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
        Ok(Task {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            status: TaskStatus::from_str(&row.get::<_, String>(3)?).unwrap(),
            priority: TaskPriority::from_str(&row.get::<_, String>(4)?).unwrap(),
            phase: row.get(5)?,
            tags: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
            created_at: row.get::<_, String>(7)?.parse().unwrap(),
            updated_at: row.get::<_, String>(8)?.parse().unwrap(),
        })
    }

    // ── File Summaries ──

    pub fn upsert_file_summary(
        &self,
        path: &str,
        summary: &str,
        key_types: &[String],
        dependencies: &[String],
        tags: &[String],
        content_hash: &str,
    ) -> Result<FileSummary> {
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        let key_types_json = serde_json::to_string(key_types)?;
        let deps_json = serde_json::to_string(dependencies)?;
        let tags_json = serde_json::to_string(tags)?;

        // Check if file already exists
        let existing_id: Option<String> = self
            .conn
            .query_row(
                "SELECT id FROM file_summaries WHERE path = ?1",
                params![path],
                |row| row.get(0),
            )
            .ok();

        let id = existing_id.unwrap_or_else(|| Uuid::new_v4().to_string());

        self.conn.execute(
            "INSERT INTO file_summaries (id, path, summary, key_types, dependencies, tags, content_hash, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)
             ON CONFLICT(path) DO UPDATE SET summary=?3, key_types=?4, dependencies=?5, tags=?6, content_hash=?7, updated_at=?8",
            params![id, path, summary, key_types_json, deps_json, tags_json, content_hash, now_str],
        )?;

        // Update FTS index
        self.conn.execute(
            "DELETE FROM search_index WHERE entity_id = ?1 AND entity_type = 'file'",
            params![id],
        ).ok(); // Ignore if not exists
        let search_content = format!("{path} {summary}");
        self.conn.execute(
            "INSERT INTO search_index (entity_id, entity_type, body) VALUES (?1, 'file', ?2)",
            params![id, search_content],
        )?;

        Ok(FileSummary {
            id,
            path: path.to_string(),
            summary: summary.to_string(),
            key_types: key_types.to_vec(),
            dependencies: dependencies.to_vec(),
            tags: tags.to_vec(),
            content_hash: content_hash.to_string(),
            created_at: now,
            updated_at: now,
        })
    }

    pub fn list_file_summaries(&self) -> Result<Vec<FileSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, summary, key_types, dependencies, tags, content_hash, created_at, updated_at FROM file_summaries ORDER BY path",
        )?;
        let rows = stmt.query_map([], Self::map_file_summary)?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    pub fn get_file_summary_by_path(&self, path: &str) -> Result<Option<FileSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, summary, key_types, dependencies, tags, content_hash, created_at, updated_at FROM file_summaries WHERE path = ?1",
        )?;
        let mut rows = stmt.query_map(params![path], Self::map_file_summary)?;
        match rows.next() {
            Some(Ok(f)) => Ok(Some(f)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    fn map_file_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<FileSummary> {
        Ok(FileSummary {
            id: row.get(0)?,
            path: row.get(1)?,
            summary: row.get(2)?,
            key_types: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
            dependencies: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
            tags: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
            content_hash: row.get(6)?,
            created_at: row.get::<_, String>(7)?.parse().unwrap(),
            updated_at: row.get::<_, String>(8)?.parse().unwrap(),
        })
    }

    // ── Sessions ──

    pub fn start_session(&self, agent: &str, goal: &str, tags: &[String]) -> Result<Session> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        let tags_json = serde_json::to_string(tags)?;

        self.conn.execute(
            "INSERT INTO sessions (id, agent, goal, tags, started_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, agent, goal, tags_json, now_str],
        )?;

        let search_content = format!("{agent} {goal}");
        self.conn.execute(
            "INSERT INTO search_index (entity_id, entity_type, body) VALUES (?1, 'session', ?2)",
            params![id, search_content],
        )?;

        Ok(Session {
            id,
            agent: agent.to_string(),
            goal: goal.to_string(),
            handoff: None,
            tags: tags.to_vec(),
            started_at: now,
            ended_at: None,
        })
    }

    pub fn end_session(&self, id: &str, handoff: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let affected = self.conn.execute(
            "UPDATE sessions SET handoff = ?1, ended_at = ?2 WHERE id = ?3",
            params![handoff, now, id],
        )?;
        if affected == 0 {
            anyhow::bail!("session not found: {id}");
        }

        // Update FTS with handoff content
        self.conn.execute(
            "DELETE FROM search_index WHERE entity_id = ?1 AND entity_type = 'session'",
            params![id],
        ).ok();
        let session = self.get_session(id)?.unwrap();
        let search_content = format!("{} {} {handoff}", session.agent, session.goal);
        self.conn.execute(
            "INSERT INTO search_index (entity_id, entity_type, body) VALUES (?1, 'session', ?2)",
            params![id, search_content],
        )?;

        Ok(())
    }

    pub fn get_active_session(&self) -> Result<Option<Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, agent, goal, handoff, tags, started_at, ended_at FROM sessions WHERE ended_at IS NULL ORDER BY started_at DESC LIMIT 1",
        )?;
        let mut rows = stmt.query_map([], Self::map_session)?;
        match rows.next() {
            Some(Ok(s)) => Ok(Some(s)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn get_session(&self, id: &str) -> Result<Option<Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, agent, goal, handoff, tags, started_at, ended_at FROM sessions WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], Self::map_session)?;
        match rows.next() {
            Some(Ok(s)) => Ok(Some(s)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn list_sessions(&self, limit: usize) -> Result<Vec<Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, agent, goal, handoff, tags, started_at, ended_at FROM sessions ORDER BY started_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], Self::map_session)?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    fn map_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<Session> {
        Ok(Session {
            id: row.get(0)?,
            agent: row.get(1)?,
            goal: row.get(2)?,
            handoff: row.get(3)?,
            tags: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
            started_at: row.get::<_, String>(5)?.parse().unwrap(),
            ended_at: row.get::<_, Option<String>>(6)?.and_then(|s| s.parse().ok()),
        })
    }

    // ── Search ──

    pub fn search(&self, query: &str, entity_type: Option<&str>) -> Result<Vec<(String, String)>> {
        let mut results = Vec::new();
        if let Some(et) = entity_type {
            let mut stmt = self.conn.prepare(
                "SELECT entity_id, entity_type FROM search_index WHERE search_index MATCH ?1 AND entity_type = ?2 ORDER BY rank",
            )?;
            let rows = stmt.query_map(params![query, et], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            for r in rows {
                results.push(r?);
            }
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT entity_id, entity_type FROM search_index WHERE search_index MATCH ?1 ORDER BY rank",
            )?;
            let rows = stmt.query_map(params![query], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            for r in rows {
                results.push(r?);
            }
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests;
