use anyhow::Result;
use sha2::{Digest, Sha256};

use super::{parse_csv, FileAction};
use crate::cli::init::open_store;

pub fn run(action: FileAction) -> Result<()> {
    let store = open_store()?;

    match action {
        FileAction::Summarize {
            path,
            summary,
            key_types,
            deps,
            tags,
        } => {
            let key_types = parse_csv(&key_types);
            let deps = parse_csv(&deps);
            let tags = parse_csv(&tags);

            // Compute content hash if file exists
            let content_hash = if std::path::Path::new(&path).exists() {
                let content = std::fs::read(&path)?;
                let hash = Sha256::digest(&content);
                format!("{:x}", hash)
            } else {
                "unknown".to_string()
            };

            let f = store.upsert_file_summary(&path, &summary, &key_types, &deps, &tags, &content_hash)?;
            println!("File summary saved: {}", f.path);
        }
        FileAction::List => {
            let files = store.list_file_summaries()?;
            if files.is_empty() {
                println!("No file summaries found.");
                return Ok(());
            }
            for f in &files {
                println!("  {} - {}", f.path, f.summary);
            }
            println!("\n{} file(s)", files.len());
        }
        FileAction::Check { path } => {
            match store.get_file_summary_by_path(&path)? {
                Some(f) => {
                    if std::path::Path::new(&path).exists() {
                        let content = std::fs::read(&path)?;
                        let hash = Sha256::digest(&content);
                        let current_hash = format!("{:x}", hash);
                        if current_hash == f.content_hash {
                            println!("{path}: up to date");
                        } else {
                            println!("{path}: STALE (file changed since last summary)");
                        }
                    } else {
                        println!("{path}: file not found on disk");
                    }
                }
                None => {
                    println!("{path}: no summary recorded");
                }
            }
        }
    }
    Ok(())
}
