use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::cli::init::open_store;
use crate::models::*;
use crate::storage::Store;

#[derive(Serialize, Deserialize)]
pub struct ExportData {
    pub version: u32,
    pub project_meta: Option<ProjectMeta>,
    pub decisions: Vec<Decision>,
    pub tasks: Vec<Task>,
    pub file_summaries: Vec<FileSummary>,
    pub sessions: Vec<Session>,
}

pub fn run_export(path: Option<&str>) -> Result<()> {
    let store = open_store()?;
    let data = export_all(&store)?;
    let json = serde_json::to_string_pretty(&data)?;

    match path {
        Some(p) => {
            std::fs::write(p, &json)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o600))?;
            }
            println!("Exported to {p}");
        }
        None => {
            println!("{json}");
        }
    }

    let stats = format!(
        "{} decisions, {} tasks, {} files, {} sessions",
        data.decisions.len(),
        data.tasks.len(),
        data.file_summaries.len(),
        data.sessions.len(),
    );
    eprintln!("Exported: {stats}");
    Ok(())
}

pub fn run_import(path: &str) -> Result<()> {
    let store = open_store()?;
    let json = std::fs::read_to_string(path)?;
    let data: ExportData = serde_json::from_str(&json)?;

    if data.version != 1 {
        anyhow::bail!("unsupported export version: {}", data.version);
    }

    import_all(&store, &data)?;

    let stats = format!(
        "{} decisions, {} tasks, {} files, {} sessions",
        data.decisions.len(),
        data.tasks.len(),
        data.file_summaries.len(),
        data.sessions.len(),
    );
    println!("Imported from {path}: {stats}");
    if let Some(ref meta) = data.project_meta {
        println!("  Project: {}", meta.name);
    }
    Ok(())
}

fn export_all(store: &Store) -> Result<ExportData> {
    Ok(ExportData {
        version: 1,
        project_meta: store.get_project_meta()?,
        decisions: store.list_decisions(None)?,
        tasks: store.list_tasks(None)?,
        file_summaries: store.list_file_summaries()?,
        sessions: store.list_sessions(1000)?,
    })
}

fn import_all(store: &Store, data: &ExportData) -> Result<()> {
    if let Some(ref meta) = data.project_meta {
        store.import_project_meta(meta)?;
    }
    for d in &data.decisions {
        store.import_decision(d)?;
    }
    for t in &data.tasks {
        store.import_task(t)?;
    }
    for f in &data.file_summaries {
        store.import_file_summary(f)?;
    }
    for s in &data.sessions {
        store.import_session(s)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_import_roundtrip() {
        let store1 = Store::open_memory().unwrap();
        store1
            .set_project_meta("TestExport", "export test")
            .unwrap();
        store1
            .add_decision("Dec1", "ctx", "decided", &[], &["tag1".into()])
            .unwrap();
        let t = store1
            .add_task("Task1", "desc", &TaskPriority::High, Some("P1"), &[])
            .unwrap();
        store1
            .update_task_status(&t.id, &TaskStatus::InProgress)
            .unwrap();
        store1
            .upsert_file_summary("src/main.rs", "entry", &["main".into()], &[], &[], "hash1")
            .unwrap();
        let s = store1.start_session("claude", "build", &[]).unwrap();
        store1.end_session(&s.id, "done, next: tests").unwrap();

        let data = export_all(&store1).unwrap();
        let json = serde_json::to_string(&data).unwrap();

        // Import into a fresh store
        let store2 = Store::open_memory().unwrap();
        let imported: ExportData = serde_json::from_str(&json).unwrap();
        import_all(&store2, &imported).unwrap();

        // Verify
        let meta = store2.get_project_meta().unwrap().unwrap();
        assert_eq!(meta.name, "TestExport");

        let decisions = store2.list_decisions(None).unwrap();
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].title, "Dec1");
        assert_eq!(decisions[0].tags, vec!["tag1"]);

        let tasks = store2.list_tasks(None).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].status, TaskStatus::InProgress);
        assert_eq!(tasks[0].phase, Some("P1".into()));

        let files = store2.list_file_summaries().unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].key_types, vec!["main"]);

        let sessions = store2.list_sessions(10).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].handoff.as_deref(), Some("done, next: tests"));
    }
}
