use anyhow::Result;

use crate::cli::init::open_store;

pub fn run(query: &str, entity_type: Option<&str>) -> Result<()> {
    let store = open_store()?;

    let results = store.search(query, entity_type)?;
    if results.is_empty() {
        println!("No results for: {query}");
        return Ok(());
    }

    for (id, etype) in &results {
        let short_id = &id[..8.min(id.len())];
        match etype.as_str() {
            "decision" => {
                if let Some(d) = store.get_decision(id)? {
                    println!("[decision:{short_id}] {} - {}", d.title, d.decision);
                }
            }
            "task" => {
                if let Some(t) = store.get_task(id)? {
                    println!(
                        "[task:{short_id}] ({}) {} ",
                        t.status.as_str(),
                        t.title,
                    );
                }
            }
            "file" => {
                if let Some(f) = store.get_file_summary_by_path(id).ok().flatten() {
                    println!("[file] {} - {}", f.path, f.summary);
                } else {
                    println!("[file:{short_id}] (details unavailable)");
                }
            }
            "session" => {
                if let Some(s) = store.get_session(id)? {
                    let status = if s.ended_at.is_some() { "ended" } else { "active" };
                    println!("[session:{short_id}] ({status}) {} - {}", s.agent, s.goal);
                }
            }
            _ => {
                println!("[{etype}:{short_id}]");
            }
        }
    }
    println!("\n{} result(s)", results.len());
    Ok(())
}
