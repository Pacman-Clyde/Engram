use anyhow::Result;

use super::{parse_csv, TaskAction};
use crate::cli::init::open_store;
use crate::models::{TaskPriority, TaskStatus};

pub fn run(action: TaskAction) -> Result<()> {
    let store = open_store()?;

    match action {
        TaskAction::Add {
            title,
            description,
            priority,
            phase,
            tags,
        } => {
            let priority = TaskPriority::from_str(&priority)?;
            let tags = parse_csv(&tags);
            let t = store.add_task(&title, &description, &priority, phase.as_deref(), &tags)?;
            println!("Task created: {}", t.id);
            println!("  Title: {}", t.title);
            println!("  Priority: {}", t.priority.as_str());
            if let Some(ref phase) = t.phase {
                println!("  Phase: {phase}");
            }
        }
        TaskAction::Update { id, status } => {
            let status = TaskStatus::from_str(&status)?;
            // Find task by prefix
            let tasks = store.list_tasks(None)?;
            let found = tasks.iter().find(|t| t.id.starts_with(&id));
            match found {
                Some(t) => {
                    store.update_task_status(&t.id, &status)?;
                    println!("Task {} updated to {}", &t.id[..8], status.as_str());
                }
                None => {
                    anyhow::bail!("task not found with ID prefix: {id}");
                }
            }
        }
        TaskAction::List { status } => {
            let status_filter = status
                .as_deref()
                .map(TaskStatus::from_str)
                .transpose()?;
            let tasks = store.list_tasks(status_filter.as_ref())?;
            if tasks.is_empty() {
                println!("No tasks found.");
                return Ok(());
            }
            for t in &tasks {
                let short_id = &t.id[..8];
                let phase = t.phase.as_deref().unwrap_or("-");
                println!(
                    "[{short_id}] ({}/{}) {} [{}]",
                    t.status.as_str(),
                    t.priority.as_str(),
                    t.title,
                    phase,
                );
            }
            println!("\n{} task(s)", tasks.len());
        }
    }
    Ok(())
}
