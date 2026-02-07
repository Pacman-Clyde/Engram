use std::io;

use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{Shell, generate};

use super::Cli;

pub fn run(shell: &str) -> Result<()> {
    let shell: Shell = shell
        .parse()
        .map_err(|_| anyhow::anyhow!("unsupported shell: {shell}. Use: bash, zsh, fish, elvish, powershell"))?;

    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "engram", &mut io::stdout());
    Ok(())
}
