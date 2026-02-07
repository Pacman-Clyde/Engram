use std::fs;

use anyhow::{Context, Result};

use crate::storage::Store;

#[cfg(unix)]
fn set_restrictive_permissions(path: &std::path::Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let perms = if path.is_dir() {
        fs::Permissions::from_mode(0o700)
    } else {
        fs::Permissions::from_mode(0o600)
    };
    fs::set_permissions(path, perms).context("failed to set permissions")?;
    Ok(())
}

#[cfg(not(unix))]
fn set_restrictive_permissions(_path: &std::path::Path) -> Result<()> {
    Ok(())
}

pub fn open_store() -> Result<Store> {
    let db_path = crate::find_engram_db()?;
    Store::open(&db_path)
}

pub fn run(name: Option<String>, description: Option<String>) -> Result<()> {
    let dir = std::path::PathBuf::from(".engram");
    if dir.exists() {
        println!("engram already initialized in .engram/");
        return Ok(());
    }

    fs::create_dir_all(&dir).context("failed to create .engram directory")?;
    set_restrictive_permissions(&dir)?;
    let db_path = dir.join("memory.db");
    let store = Store::open(&db_path).context("failed to create database")?;
    set_restrictive_permissions(&db_path)?;

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
