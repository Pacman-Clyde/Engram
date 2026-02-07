use std::str::FromStr;

use anyhow::Result;

use super::{parse_csv, DecisionAction};
use crate::cli::init::open_store;
use crate::models::DecisionStatus;

pub fn run(action: DecisionAction) -> Result<()> {
    let store = open_store()?;

    match action {
        DecisionAction::Add {
            title,
            context,
            decision,
            alternatives,
            tags,
        } => {
            let alts = parse_csv(&alternatives);
            let tags = parse_csv(&tags);
            let d = store.add_decision(&title, &context, &decision, &alts, &tags)?;
            println!("Decision recorded: {}", d.id);
            println!("  Title: {}", d.title);
        }
        DecisionAction::List { status } => {
            let status_filter = status
                .as_deref()
                .map(DecisionStatus::from_str)
                .transpose()?;
            let decisions = store.list_decisions(status_filter.as_ref())?;
            if decisions.is_empty() {
                println!("No decisions found.");
                return Ok(());
            }
            for d in &decisions {
                let short_id = &d.id[..8.min(d.id.len())];
                println!(
                    "[{short_id}] ({}) {} - {}",
                    d.status.as_str(),
                    d.title,
                    d.decision
                );
            }
            println!("\n{} decision(s)", decisions.len());
        }
        DecisionAction::Show { id } => {
            let decisions = store.list_decisions(None)?;
            let found = decisions.iter().find(|d| d.id.starts_with(&id));
            match found {
                Some(d) => {
                    println!("# {}", d.title);
                    println!("ID: {}", d.id);
                    println!("Status: {}", d.status.as_str());
                    println!("Created: {}", d.created_at.format("%Y-%m-%d %H:%M"));
                    println!("\n## Context\n{}", d.context);
                    println!("\n## Decision\n{}", d.decision);
                    if !d.alternatives.is_empty() {
                        println!("\n## Alternatives Considered");
                        for alt in &d.alternatives {
                            println!("- {alt}");
                        }
                    }
                    if !d.tags.is_empty() {
                        println!("\nTags: {}", d.tags.join(", "));
                    }
                }
                None => {
                    anyhow::bail!("decision not found with ID prefix: {id}");
                }
            }
        }
    }
    Ok(())
}
