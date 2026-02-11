use super::*;

fn test_store() -> Store {
    Store::open_memory().expect("failed to create in-memory store")
}

#[test]
fn test_project_meta() {
    let store = test_store();

    assert!(store.get_project_meta().unwrap().is_none());

    store
        .set_project_meta("TestProject", "A test project")
        .unwrap();
    let meta = store.get_project_meta().unwrap().unwrap();
    assert_eq!(meta.name, "TestProject");
    assert_eq!(meta.description, "A test project");
    assert!(meta.stack.is_empty());

    store
        .update_project_meta_stack(&["Rust".into(), "SQLite".into()])
        .unwrap();
    let meta = store.get_project_meta().unwrap().unwrap();
    assert_eq!(meta.stack, vec!["Rust", "SQLite"]);

    store
        .update_project_meta_conventions(&["snake_case".into()])
        .unwrap();
    let meta = store.get_project_meta().unwrap().unwrap();
    assert_eq!(meta.conventions, vec!["snake_case"]);

    // Overwrite project meta
    store.set_project_meta("NewName", "Updated desc").unwrap();
    let meta = store.get_project_meta().unwrap().unwrap();
    assert_eq!(meta.name, "NewName");
}

#[test]
fn test_decisions_crud() {
    let store = test_store();

    let d1 = store
        .add_decision(
            "Use rusqlite",
            "Need local SQL",
            "rusqlite with bundled SQLite",
            &["sqlx".into(), "sled".into()],
            &["architecture".into()],
        )
        .unwrap();
    assert_eq!(d1.title, "Use rusqlite");
    assert_eq!(d1.status, DecisionStatus::Active);

    let _d2 = store
        .add_decision("Use clap", "Need CLI parsing", "clap derive API", &[], &[])
        .unwrap();

    // List all
    let all = store.list_decisions(None).unwrap();
    assert_eq!(all.len(), 2);

    // List by status
    let active = store.list_decisions(Some(&DecisionStatus::Active)).unwrap();
    assert_eq!(active.len(), 2);

    let superseded = store
        .list_decisions(Some(&DecisionStatus::Superseded))
        .unwrap();
    assert_eq!(superseded.len(), 0);

    // Get by ID
    let fetched = store.get_decision(&d1.id).unwrap().unwrap();
    assert_eq!(fetched.title, "Use rusqlite");
    assert_eq!(fetched.alternatives, vec!["sqlx", "sled"]);

    // Not found
    assert!(store.get_decision("nonexistent").unwrap().is_none());
}

#[test]
fn test_tasks_crud() {
    let store = test_store();

    let t1 = store
        .add_task(
            "Implement storage",
            "Build SQLite CRUD layer",
            &TaskPriority::High,
            Some("Phase 1"),
            &["backend".into()],
        )
        .unwrap();
    assert_eq!(t1.status, TaskStatus::Todo);
    assert_eq!(t1.priority, TaskPriority::High);
    assert_eq!(t1.phase, Some("Phase 1".into()));

    let _t2 = store
        .add_task("Write tests", "", &TaskPriority::Medium, None, &[])
        .unwrap();

    // List all
    let all = store.list_tasks(None).unwrap();
    assert_eq!(all.len(), 2);

    // List by status
    let todos = store.list_tasks(Some(&TaskStatus::Todo)).unwrap();
    assert_eq!(todos.len(), 2);

    // Update status
    store
        .update_task_status(&t1.id, &TaskStatus::InProgress)
        .unwrap();
    let updated = store.get_task(&t1.id).unwrap().unwrap();
    assert_eq!(updated.status, TaskStatus::InProgress);

    // Filter shows correctly after update
    let todos = store.list_tasks(Some(&TaskStatus::Todo)).unwrap();
    assert_eq!(todos.len(), 1);

    let in_prog = store.list_tasks(Some(&TaskStatus::InProgress)).unwrap();
    assert_eq!(in_prog.len(), 1);
    assert_eq!(in_prog[0].id, t1.id);

    // Not found
    let err = store.update_task_status("nonexistent", &TaskStatus::Done);
    assert!(err.is_err());
}

#[test]
fn test_file_summaries() {
    let store = test_store();

    let f1 = store
        .upsert_file_summary(
            "src/main.rs",
            "Entry point",
            &["main".into()],
            &[],
            &[],
            "abc123",
        )
        .unwrap();
    assert_eq!(f1.path, "src/main.rs");

    // Upsert same path updates
    let f1_updated = store
        .upsert_file_summary(
            "src/main.rs",
            "Updated entry point",
            &["main".into(), "cli".into()],
            &["clap".into()],
            &[],
            "def456",
        )
        .unwrap();
    assert_eq!(f1_updated.summary, "Updated entry point");

    let list = store.list_file_summaries().unwrap();
    assert_eq!(list.len(), 1); // Still one, not two

    let fetched = store
        .get_file_summary_by_path("src/main.rs")
        .unwrap()
        .unwrap();
    assert_eq!(fetched.summary, "Updated entry point");
    assert_eq!(fetched.content_hash, "def456");
    let fetched_by_id = store.get_file_summary(&fetched.id).unwrap().unwrap();
    assert_eq!(fetched_by_id.path, "src/main.rs");

    assert!(store
        .get_file_summary_by_path("nonexistent.rs")
        .unwrap()
        .is_none());
    assert!(store.get_file_summary("nonexistent-id").unwrap().is_none());
}

#[test]
fn test_sessions() {
    let store = test_store();

    let s1 = store
        .start_session("claude", "Implement Phase 1", &["dev".into()])
        .unwrap();
    assert!(s1.ended_at.is_none());
    assert!(s1.handoff.is_none());

    // Active session
    let active = store.get_active_session().unwrap().unwrap();
    assert_eq!(active.id, s1.id);

    // End session
    store
        .end_session(&s1.id, "Completed CRUD. Next: implement FTS5.")
        .unwrap();
    let ended = store.get_session(&s1.id).unwrap().unwrap();
    assert!(ended.ended_at.is_some());
    assert_eq!(
        ended.handoff.as_deref(),
        Some("Completed CRUD. Next: implement FTS5.")
    );

    // No active session now
    assert!(store.get_active_session().unwrap().is_none());

    // List sessions
    let s2 = store.start_session("gpt", "Review code", &[]).unwrap();
    let sessions = store.list_sessions(10).unwrap();
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].id, s2.id); // Most recent first

    // Error on nonexistent
    assert!(store.end_session("nonexistent", "bye").is_err());
}

#[test]
fn test_search() {
    let store = test_store();

    store
        .add_decision(
            "Use rusqlite",
            "Need SQL database",
            "rusqlite bundled",
            &[],
            &[],
        )
        .unwrap();
    store
        .add_task(
            "Build storage",
            "Implement SQLite layer",
            &TaskPriority::High,
            None,
            &[],
        )
        .unwrap();

    // Search across all types
    let results = store.search("rusqlite", None).unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].1, "decision");

    // Search with type filter
    let results = store.search("storage", Some("task")).unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].1, "task");

    // No results
    let results = store.search("nonexistentterm12345", None).unwrap();
    assert!(results.is_empty());
}
