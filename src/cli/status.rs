use anyhow::Result;

use crate::cli::init::open_store;
use crate::models::TaskStatus;

pub fn run() -> Result<()> {
    let store = open_store()?;

    let meta = store.get_project_meta()?;
    let tasks = store.list_tasks(None)?;
    let decisions = store.list_decisions(None)?;
    let files = store.list_file_summaries()?;
    let sessions = store.list_sessions(5)?;
    let active_session = store.get_active_session()?;

    // Project header
    if let Some(ref m) = meta {
        println!("=== {} ===", m.name);
        if !m.description.is_empty() {
            println!("{}", m.description);
        }
        if !m.stack.is_empty() {
            println!("Stack: {}", m.stack.join(", "));
        }
    } else {
        println!("=== (unnamed project) ===");
    }
    println!();

    // Active session
    if let Some(ref s) = active_session {
        println!("Session: {} - {} (active)", s.agent, s.goal);
    } else if let Some(last) = sessions.first() {
        print!("Last session: {} - {}", last.agent, last.goal);
        if let Some(ref handoff) = last.handoff {
            let preview = if handoff.chars().count() > 60 {
                let truncated: String = handoff.chars().take(60).collect();
                format!("{truncated}...")
            } else {
                handoff.clone()
            };
            print!(" | Handoff: {preview}");
        }
        println!();
    }
    println!();

    // Task counts
    let mut todo = 0;
    let mut in_progress = 0;
    let mut done = 0;
    let mut blocked = 0;
    for t in &tasks {
        match t.status {
            TaskStatus::Todo => todo += 1,
            TaskStatus::InProgress => in_progress += 1,
            TaskStatus::Done => done += 1,
            TaskStatus::Blocked => blocked += 1,
        }
    }
    println!(
        "Tasks: {} total | {} in-progress | {} todo | {} done | {} blocked",
        tasks.len(),
        in_progress,
        todo,
        done,
        blocked,
    );

    // In-progress tasks
    let active_tasks: Vec<_> = tasks
        .iter()
        .filter(|t| matches!(t.status, TaskStatus::InProgress))
        .collect();
    for t in &active_tasks {
        let phase = t
            .phase
            .as_deref()
            .map(|p| format!(" [{p}]"))
            .unwrap_or_default();
        println!("  > {}{phase}", t.title);
    }

    println!(
        "Decisions: {} active",
        decisions
            .iter()
            .filter(|d| d.status == crate::models::DecisionStatus::Active)
            .count()
    );
    println!("Files: {} summarized", files.len());
    println!("Sessions: {} recorded", sessions.len());

    Ok(())
}
