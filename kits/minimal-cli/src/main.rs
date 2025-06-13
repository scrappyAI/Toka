use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    // Delegate to toka-runtime CLI
    let args = toka_runtime::cli::parse_args()?;
    toka_runtime::cli::execute_command(args).await?;

    Ok(())
} 