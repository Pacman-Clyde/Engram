use std::str::FromStr;

use anyhow::Result;

use crate::cli::init::open_store;
use crate::engine;
use crate::models::{ContextLevel, ContextRole};

pub fn run(role: &str, level: &str) -> Result<()> {
    let store = open_store()?;
    let role = ContextRole::from_str(role)?;
    let level = ContextLevel::from_str(level)?;

    let output = engine::generate_context(&store, &role, &level)?;
    println!("{}", output.markdown);
    eprintln!(
        "--- ~{} tokens ({}/{})",
        output.estimated_tokens,
        output.role.as_str(),
        output.level.as_str()
    );
    Ok(())
}
