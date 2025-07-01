//! Toka CLI – command-line interface skeleton.
//!
//! Phase-4 deliverable: provides a minimal CLI frontend to interact with
//! agents, tools, and the vault.  Each command prints a placeholder for now
//! and will be wired up to `toka-runtime` functionality in later phases.
//!
//! Usage examples (once installed):
//!   $ toka agent list
//!   $ toka agent new "super-agent"
//!   $ toka tool run echo --payload "Hello"
//!   $ toka vault get user/session
//!
//! Build & run in debug mode:
//!   cargo run -p toka-cli -- agent list
//!
//! The CLI uses `clap` derive macros and an async `main` entry-point so we can
//! later await runtime operations.

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json::json;
use std::sync::Arc;

use toka_types::{EntityId, Operation, Message};
use toka_kernel::{Kernel, WorldState};
use toka_auth::{TokenValidator, Claims};
use toka_events::bus::{InMemoryBus};
use async_trait::async_trait;

#[derive(Parser, Debug)]
#[command(name = "toka-cli", version, about = "Toka OS demo CLI", author = "Toka Project")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Mint new asset supply and credit an entity
    Mint {
        /// Asset entity ID
        #[arg(long)]
        asset: u128,
        /// Recipient entity ID
        #[arg(long)]
        to: u128,
        /// Amount to mint
        #[arg(long)]
        amount: u64,
    },
    /// Transfer funds between entities
    Transfer {
        #[arg(long)]
        from: u128,
        #[arg(long)]
        to: u128,
        #[arg(long)]
        amount: u64,
    },
    /// Query current balance for an entity
    Balance {
        #[arg(long)]
        entity: u128,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Boot demo kernel (allow-all auth, in-memory bus)
    let kernel = bootstrap_kernel();

    match cli.cmd {
        Commands::Mint { asset, to, amount } => {
            let msg = Message {
                origin: EntityId(to),
                capability: "allow".into(),
                op: Operation::MintAsset { asset: EntityId(asset), to: EntityId(to), amount },
            };
            let ev = kernel.submit(msg).await?;
            println!("{}", serde_json::to_string_pretty(&ev)?);
        }
        Commands::Transfer { from, to, amount } => {
            let msg = Message {
                origin: EntityId(from),
                capability: "allow".into(),
                op: Operation::TransferFunds { from: EntityId(from), to: EntityId(to), amount },
            };
            let ev = kernel.submit(msg).await?;
            println!("{}", serde_json::to_string_pretty(&ev)?);
        }
        Commands::Balance { entity } => {
            let state_arc = kernel.state_ptr();
            let state = state_arc.read().await;
            let bal = state.balances.get(&EntityId(entity)).copied().unwrap_or(0);
            println!("{}", json!({ "entity": entity, "balance": bal }).to_string());
        }
    }

    Ok(())
}

//──────────────────── helper bootstrap ────────────────────

fn bootstrap_kernel() -> Kernel {
    let auth = Arc::new(AllowAllValidator);
    let bus = Arc::new(InMemoryBus::default());
    Kernel::new(WorldState::default(), auth, bus)
}

struct AllowAllValidator;

#[async_trait]
impl TokenValidator for AllowAllValidator {
    async fn validate(&self, _raw: &str) -> std::result::Result<Claims, toka_auth::Error> {
        Ok(Claims {
            sub: "cli".into(),
            vault: "demo".into(),
            permissions: vec!["*".into()],
            iat: 0,
            exp: u64::MAX,
            jti: "cli".into(),
        })
    }
}
