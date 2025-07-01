#![forbid(unsafe_code)]
#![warn(missing_docs)]
//!
//! **toka-cli** – Developer command-line interface for **Toka OS**.
//!
//! The CLI is a thin, synchronous wrapper around an in-memory [`toka-kernel`](../../crates/toka-kernel)
//! instance.  Its primary goal is to provide **human-oriented ergonomics** for exploring the kernel
//! opcodes and for driving integration tests.
//!
//! It purposefully avoids introducing network stacks or persistent storage – everything runs
//! in-process so that the command latency & behaviour mirrors the deterministic kernel semantics.
//!
//! Example (mint asset then query balance):
//! ```bash
//! toka-cli mint --asset 1 --to 42 --amount 1000
//! toka-cli balance --entity 42
//! ```
//!
//! The CLI will progress alongside the roadmap; future milestones will add sub-commands for agent
//! management and event inspection.

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json::json;
use std::sync::Arc;

use toka_types::{EntityId, Operation, Message};
use toka_kernel::{Kernel, WorldState};
use toka_auth::{TokenValidator, Claims};
use toka_events::bus::InMemoryBus;
use async_trait::async_trait;

/// Root CLI parser (exposed so tests can construct commands manually).
#[derive(Parser, Debug)]
#[command(name = "toka-cli", version, about = "Toka OS demo CLI", author = "Toka Project")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Commands,
}

/// High-level commands understood by the CLI.
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
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

/// Lightweight wrapper that owns a kernel instance so we can reuse state across
/// multiple command executions (integration tests depend on this).
pub struct CliApp {
    kernel: Kernel,
}

impl CliApp {
    /// Create a new in-memory CLI runtime (auth validator always allows).
    pub fn new() -> Self {
        Self { kernel: bootstrap_kernel() }
    }

    /// Execute a single CLI `command` and **return the pretty-printed JSON**
    /// produced by the original CLI stdout.
    pub async fn execute(&self, command: Commands) -> Result<String> {
        match command {
            Commands::Mint { asset, to, amount } => {
                let msg = Message {
                    origin: EntityId(to),
                    capability: "allow".into(),
                    op: Operation::MintAsset { asset: EntityId(asset), to: EntityId(to), amount },
                };
                let ev = self.kernel.submit(msg).await?;
                Ok(serde_json::to_string_pretty(&ev)?)
            }
            Commands::Transfer { from, to, amount } => {
                let msg = Message {
                    origin: EntityId(from),
                    capability: "allow".into(),
                    op: Operation::TransferFunds { from: EntityId(from), to: EntityId(to), amount },
                };
                let ev = self.kernel.submit(msg).await?;
                Ok(serde_json::to_string_pretty(&ev)?)
            }
            Commands::Balance { entity } => {
                let state_arc = self.kernel.state_ptr();
                let state = state_arc.read().await;
                let bal = state.balances.get(&EntityId(entity)).copied().unwrap_or(0);
                Ok(json!({ "entity": entity, "balance": bal }).to_string())
            }
        }
    }
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