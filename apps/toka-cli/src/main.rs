//! Toka CLI – thin binary wrapper that delegates all logic to the `toka-cli`
//! library so integration tests can call the same code without spawning a
//! separate process.

use anyhow::Result;
use toka_cli::{Cli, CliApp};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let app = CliApp::new();
    let out = app.execute(cli.cmd).await?;
    println!("{out}");
    Ok(())
}
