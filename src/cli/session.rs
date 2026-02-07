use anyhow::Result;

use super::{parse_csv, SessionAction};
use crate::cli::init::open_store;

pub fn run(action: SessionAction) -> Result<()> {
    let store = open_store()?;

    match action {
        SessionAction::Start { agent, goal, tags } => {
            // Check for active session
            if let Some(active) = store.get_active_session()? {
                anyhow::bail!(
                    "session already active: {} ({}). End it first with `engram session end`",
                    &active.id[..8.min(active.id.len())],
                    active.goal
                );
            }
            let tags = parse_csv(&tags);
            let s = store.start_session(&agent, &goal, &tags)?;
            println!("Session started: {}", &s.id[..8.min(s.id.len())]);
            println!("  Agent: {}", s.agent);
            println!("  Goal: {}", s.goal);
        }
        SessionAction::End { handoff } => {
            let active = store.get_active_session()?;
            match active {
                Some(s) => {
                    store.end_session(&s.id, &handoff)?;
                    println!("Session ended: {}", &s.id[..8.min(s.id.len())]);
                    println!("  Handoff: {handoff}");
                }
                None => {
                    anyhow::bail!("no active session to end");
                }
            }
        }
        SessionAction::List { limit } => {
            let sessions = store.list_sessions(limit)?;
            if sessions.is_empty() {
                println!("No sessions found.");
                return Ok(());
            }
            for s in &sessions {
                let short_id = &s.id[..8.min(s.id.len())];
                let status = if s.ended_at.is_some() {
                    "ended"
                } else {
                    "active"
                };
                println!(
                    "[{short_id}] ({status}) {} - {} ({})",
                    s.agent,
                    s.goal,
                    s.started_at.format("%Y-%m-%d %H:%M"),
                );
                if let Some(ref handoff) = s.handoff {
                    let preview = if handoff.chars().count() > 80 {
                        let truncated: String = handoff.chars().take(80).collect();
                        format!("{truncated}...")
                    } else {
                        handoff.clone()
                    };
                    println!("           Handoff: {preview}");
                }
            }
            println!("\n{} session(s)", sessions.len());
        }
    }
    Ok(())
}
