use crate::models::*;
use crate::storage::Store;

use super::generate_context;

fn seed_store() -> Store {
    let store = Store::open_memory().unwrap();
    store.set_project_meta("TestProject", "A test project").unwrap();
    store
        .update_project_meta_stack(&["Rust".into(), "SQLite".into()])
        .unwrap();
    store
        .update_project_meta_conventions(&["snake_case".into(), "no unwrap in lib".into()])
        .unwrap();

    store
        .add_decision(
            "Use rusqlite",
            "Need local SQL database",
            "rusqlite with bundled SQLite",
            &["sqlx".into(), "sled".into()],
            &["architecture".into()],
        )
        .unwrap();
    store
        .add_decision(
            "Use clap",
            "Need CLI parsing",
            "clap derive API",
            &[],
            &[],
        )
        .unwrap();

    store
        .add_task(
            "Build storage",
            "Implement SQLite CRUD",
            &TaskPriority::High,
            Some("Phase 1"),
            &[],
        )
        .unwrap();
    let t2 = store
        .add_task("Write tests", "Unit tests for CRUD", &TaskPriority::Medium, None, &[])
        .unwrap();
    store.update_task_status(&t2.id, &TaskStatus::InProgress).unwrap();
    let t3 = store
        .add_task("Deploy", "Ship it", &TaskPriority::Low, None, &[])
        .unwrap();
    store.update_task_status(&t3.id, &TaskStatus::Done).unwrap();

    store
        .upsert_file_summary(
            "src/main.rs",
            "CLI entry point",
            &["main".into()],
            &["clap".into()],
            &[],
            "abc123",
        )
        .unwrap();

    let s1 = store.start_session("claude", "Build Phase 1", &[]).unwrap();
    store
        .end_session(&s1.id, "Completed CRUD. Next: FTS5 indexing.")
        .unwrap();
    store.start_session("gpt", "Review code", &[]).unwrap();

    store
}

// ── Minimal Level ──

#[test]
fn test_minimal_build() {
    let store = seed_store();
    let ctx = generate_context(&store, &ContextRole::Build, &ContextLevel::Minimal).unwrap();

    assert_eq!(ctx.role, ContextRole::Build);
    assert_eq!(ctx.level, ContextLevel::Minimal);
    assert!(ctx.markdown.contains("# TestProject"));
    assert!(ctx.markdown.contains("**Next**:"));
    // Minimal build should NOT contain decisions or file summaries
    assert!(!ctx.markdown.contains("## Decisions"));
    assert!(!ctx.markdown.contains("## Files"));
    // Should NOT contain handoff (that's resume-only at minimal)
    assert!(!ctx.markdown.contains("**Last handoff**"));
    assert!(ctx.estimated_tokens > 0);
}

#[test]
fn test_minimal_resume() {
    let store = seed_store();
    let ctx = generate_context(&store, &ContextRole::Resume, &ContextLevel::Minimal).unwrap();

    assert!(ctx.markdown.contains("**Next**:"));
    assert!(ctx.markdown.contains("**Last handoff**"));
    assert!(ctx.markdown.contains("Completed CRUD"));
}

#[test]
fn test_minimal_debug() {
    let store = seed_store();
    let ctx = generate_context(&store, &ContextRole::Debug, &ContextLevel::Minimal).unwrap();

    assert!(ctx.markdown.contains("# TestProject"));
    assert!(ctx.markdown.contains("**Next**:"));
    // No decisions at minimal
    assert!(!ctx.markdown.contains("## Decisions"));
}

#[test]
fn test_minimal_review() {
    let store = seed_store();
    let ctx = generate_context(&store, &ContextRole::Review, &ContextLevel::Minimal).unwrap();

    assert!(ctx.markdown.contains("# TestProject"));
    // No decisions at minimal level
    assert!(!ctx.markdown.contains("## Decisions"));
}

// ── Standard Level ──

#[test]
fn test_standard_build() {
    let store = seed_store();
    let ctx = generate_context(&store, &ContextRole::Build, &ContextLevel::Standard).unwrap();

    assert!(ctx.markdown.contains("# TestProject"));
    assert!(ctx.markdown.contains("**Stack**: Rust, SQLite"));
    // Build role gets decisions at standard
    assert!(ctx.markdown.contains("## Decisions"));
    assert!(ctx.markdown.contains("Use rusqlite"));
    // Gets active tasks
    assert!(ctx.markdown.contains("## Tasks"));
    assert!(ctx.markdown.contains("Build storage"));
    assert!(ctx.markdown.contains("Write tests"));
    // Done tasks filtered out at standard
    assert!(!ctx.markdown.contains("Deploy")); // the Done task
    // Last session
    assert!(ctx.markdown.contains("## Last Session"));
    // No file summaries at standard
    assert!(!ctx.markdown.contains("## Files"));
    // No recently completed (that's debug-only)
    assert!(!ctx.markdown.contains("## Recently Completed"));
}

#[test]
fn test_standard_review() {
    let store = seed_store();
    let ctx = generate_context(&store, &ContextRole::Review, &ContextLevel::Standard).unwrap();

    // Review also gets decisions
    assert!(ctx.markdown.contains("## Decisions"));
    assert!(ctx.markdown.contains("## Tasks"));
    assert!(ctx.markdown.contains("## Last Session"));
    // No recently completed
    assert!(!ctx.markdown.contains("## Recently Completed"));
}

#[test]
fn test_standard_debug() {
    let store = seed_store();
    let ctx = generate_context(&store, &ContextRole::Debug, &ContextLevel::Standard).unwrap();

    // Debug does NOT get decisions at standard
    assert!(!ctx.markdown.contains("## Decisions"));
    // Gets tasks
    assert!(ctx.markdown.contains("## Tasks"));
    // Gets recently completed (debug-specific)
    assert!(ctx.markdown.contains("## Recently Completed"));
    assert!(ctx.markdown.contains("Deploy")); // the Done task
}

#[test]
fn test_standard_resume() {
    let store = seed_store();
    let ctx = generate_context(&store, &ContextRole::Resume, &ContextLevel::Standard).unwrap();

    // Resume does NOT get decisions at standard
    assert!(!ctx.markdown.contains("## Decisions"));
    // Gets tasks and last session
    assert!(ctx.markdown.contains("## Tasks"));
    assert!(ctx.markdown.contains("## Last Session"));
}

// ── Full Level ──

#[test]
fn test_full_build() {
    let store = seed_store();
    let ctx = generate_context(&store, &ContextRole::Build, &ContextLevel::Full).unwrap();

    // Conventions
    assert!(ctx.markdown.contains("## Conventions"));
    assert!(ctx.markdown.contains("snake_case"));
    // Decisions with full rationale
    assert!(ctx.markdown.contains("## Decisions"));
    assert!(ctx.markdown.contains("### Use rusqlite"));
    assert!(ctx.markdown.contains("**Context**:"));
    assert!(ctx.markdown.contains("**Decision**:"));
    assert!(ctx.markdown.contains("**Alternatives**: sqlx, sled"));
    // All tasks including done
    assert!(ctx.markdown.contains("## Tasks"));
    assert!(ctx.markdown.contains("Deploy"));
    // File summaries for build role
    assert!(ctx.markdown.contains("## Files"));
    assert!(ctx.markdown.contains("src/main.rs"));
    assert!(ctx.markdown.contains("CLI entry point"));
    // Session history
    assert!(ctx.markdown.contains("## Session History"));
}

#[test]
fn test_full_review() {
    let store = seed_store();
    let ctx = generate_context(&store, &ContextRole::Review, &ContextLevel::Full).unwrap();

    assert!(ctx.markdown.contains("## Conventions"));
    assert!(ctx.markdown.contains("## Decisions"));
    assert!(ctx.markdown.contains("## Files"));
    assert!(ctx.markdown.contains("## Session History"));
}

#[test]
fn test_full_debug() {
    let store = seed_store();
    let ctx = generate_context(&store, &ContextRole::Debug, &ContextLevel::Full).unwrap();

    assert!(ctx.markdown.contains("## Conventions"));
    assert!(ctx.markdown.contains("## Decisions"));
    // Debug also gets files at full level
    assert!(ctx.markdown.contains("## Files"));
    assert!(ctx.markdown.contains("## Session History"));
}

#[test]
fn test_full_resume() {
    let store = seed_store();
    let ctx = generate_context(&store, &ContextRole::Resume, &ContextLevel::Full).unwrap();

    assert!(ctx.markdown.contains("## Conventions"));
    assert!(ctx.markdown.contains("## Decisions"));
    // Resume does NOT get files
    assert!(!ctx.markdown.contains("## Files"));
    // But gets session history
    assert!(ctx.markdown.contains("## Session History"));
    assert!(ctx.markdown.contains("Completed CRUD"));
}

// ── Edge Cases ──

#[test]
fn test_empty_project() {
    let store = Store::open_memory().unwrap();
    let ctx = generate_context(&store, &ContextRole::Build, &ContextLevel::Standard).unwrap();

    assert!(ctx.markdown.contains("# (unnamed project)"));
    assert!(!ctx.markdown.contains("## Decisions"));
    assert!(!ctx.markdown.contains("## Tasks"));
    assert!(!ctx.markdown.contains("## Last Session"));
}

#[test]
fn test_active_session_shown() {
    let store = seed_store();
    // seed_store leaves an active session (gpt reviewing)
    let ctx = generate_context(&store, &ContextRole::Build, &ContextLevel::Minimal).unwrap();
    assert!(ctx.markdown.contains("**Active session**: gpt"));
}

#[test]
fn test_token_estimate_scales_with_level() {
    let store = seed_store();
    let minimal = generate_context(&store, &ContextRole::Build, &ContextLevel::Minimal).unwrap();
    let standard = generate_context(&store, &ContextRole::Build, &ContextLevel::Standard).unwrap();
    let full = generate_context(&store, &ContextRole::Build, &ContextLevel::Full).unwrap();

    assert!(minimal.estimated_tokens < standard.estimated_tokens);
    assert!(standard.estimated_tokens < full.estimated_tokens);
}
