use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::storage::Store;

fn engram_dir() -> PathBuf {
    PathBuf::from(".engram")
}

pub fn find_engram_db() -> Result<PathBuf> {
    let dir = engram_dir();
    if !dir.exists() {
        anyhow::bail!("not an engram project (run `engram init` first)");
    }
    Ok(dir.join("memory.db"))
}

pub fn open_store() -> Result<Store> {
    let db_path = find_engram_db()?;
    Store::open(&db_path)
}

pub fn run(name: Option<String>, description: Option<String>) -> Result<()> {
    let dir = engram_dir();
    if dir.exists() {
        println!("engram already initialized in .engram/");
        return Ok(());
    }

    fs::create_dir_all(&dir).context("failed to create .engram directory")?;
    let db_path = dir.join("memory.db");
    let store = Store::open(&db_path).context("failed to create database")?;

    let project_name = name.unwrap_or_else(|| {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "unnamed".to_string())
    });
    let desc = description.unwrap_or_default();

    store.set_project_meta(&project_name, &desc)?;

    println!("Initialized engram in .engram/");
    println!("  Project: {project_name}");
    if !desc.is_empty() {
        println!("  Description: {desc}");
    }
    println!("  Database: .engram/memory.db");
    Ok(())
}
