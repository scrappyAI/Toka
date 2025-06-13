Below is a boiler-plate “clean-slate” template for a super-lean toka-runtime that can optionally pull in heavier sub-crates when the operator (or build script) chooses.

⸻

1 Project layout (workspace)

toka/
├─ Cargo.toml             # [workspace] – no code
├─ crates/
│   ├─ toka-core          # traits only (Agent, Tool, Driver, EventKind)
│   ├─ toka-events        # enum Event, serde impls
│   ├─ toka-runtime       # ★ this crate – lean kernel
│   │   └─ src/
│   │       ├─ lib.rs
│   │       ├─ bus.rs
│   │       ├─ runtime.rs
│   │       └─ scheduler.rs
│   ├─ toka-vault-core    # encryption, manifest (optional)
│   ├─ toka-ledger-core   # mint/burn/transfer structs (optional)
│   ├─ toka-auth-core     # password-hash, token claims (optional)
│   ├─ driver-s3          # sample platform driver (optional)
│   ├─ provider-stripe    # payment provider (optional)
│   └─ ...                # more drivers/tools/providers
└─ kits/
    └─ minimal-demo/      # CLI that depends only on toka-runtime

The runtime never depends on “backend” crates directly—only via feature flags.

⸻

2 crates/toka-runtime/Cargo.toml (lean by default)

[package]
name = "toka-runtime"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio        = { version = "1", features = ["macros", "rt-multi-thread"] }
anyhow       = "1"
serde        = { version = "1", features = ["derive"] }
serde_json   = "1"
toml         = "0.8"
tracing      = "0.1"
sled         = { version = "0.34", optional = true }      # secrets store
libloading   = { version = "0.8",  optional = true }      # plugin host

toka-core    = { path = "../toka-core" }
toka-events  = { path = "../toka-events" }

# -------- optional heavier crates ----------
toka-vault-core    = { path = "../toka-vault-core", optional = true }
toka-ledger-core   = { path = "../toka-ledger-core", optional = true }
toka-auth-core     = { path = "../toka-auth-core",  optional = true }
driver-s3          = { path = "../driver-s3",       optional = true }
provider-stripe    = { path = "../provider-stripe", optional = true }

[features]
default = []                      # pure in-memory

# built-in light helpers
secrets          = ["sled"]       # local KV for API keys
plugins          = ["libloading"] # .so/.dll hot-loading

# optional core modules
vault-core       = ["toka-vault-core"]
ledger-core      = ["toka-ledger-core"]
auth-core        = ["toka-auth-core"]

# optional drivers / providers
driver-s3        = ["driver-s3"]
provider-stripe  = ["provider-stripe"]

# convenience umbrella for a full desktop build
portable = [
  "secrets",
  "vault-core",
  "driver-s3",
  "provider-stripe"
]

Compile modes:

# ultra-lean
cargo build -p toka-runtime --release --no-default-features

# one-file “portable” bundle
cargo build -p toka-runtime --release --features portable


⸻

3 Runtime configuration file (RuntimeConfig)

3.1 config/minimal.toml

# minimal example – only runtime, no drivers
[event_bus]
buffer_size = 1024          # in-memory ring buffer
scheduler_tick_ms = 100
secret_store = "secrets.db" # created if `secrets` feature

[[agents]]
name = "Logger"             # compiled-in agent

3.2 config/portable.toml

# rich desktop build
[event_bus]
buffer_size = 4096
scheduler_tick_ms = 100
secret_store = "secrets.db"

[vault]
path = "/Users/me/TokaVault"   # requires vault-core feature

[drivers.s3]
access_key = "${AWS_KEY}"
secret_key = "${AWS_SECRET}"
bucket     = "my-assets"

[providers.stripe]
secret_key = "${STRIPE_SK}"
webhook_secret = "${STRIPE_WEBHOOK}"

[[agents]]
name = "IndexerAgent"
scan_path = "/Users/me/Content"

[[agents]]
name = "SchedulerAgent"
default_driver = "s3"


⸻

4 runtime.rs   (key excerpts)

/// Build-time re-exports so a user crate only needs `toka-runtime`
pub use toka_core::{Agent, Event, EventKind};

pub struct Runtime {
    bus: Bus,
    scheduler: Scheduler,
    registry: Registry,
}

impl Runtime {
    pub fn new(cfg: Config) -> anyhow::Result<Self> {
        let (bus, scheduler) = Bus::new(cfg.event_bus.buffer_size, cfg.scheduler_tick);
        let mut rt = Self { bus, scheduler, registry: Registry::default() };

        #[cfg(feature = "vault-core")]
        if let Some(v) = cfg.vault.as_ref() {
            let vault = toka_vault_core::FileVault::open(&v.path)?;
            rt.registry.set_vault(Box::new(vault));
        }

        #[cfg(feature = "driver-s3")]
        if let Some(s3cfg) = cfg.drivers.get("s3") {
            rt.registry.register_platform(Box::new(
                driver_s3::S3Driver::new(s3cfg.clone())?
            ));
        }

        // load agents compiled in this build
        rt.load_agents(cfg.agents)?;
        // optional plugin discovery
        #[cfg(feature = "plugins")]
        rt.load_plugins("~/.toka/plugins")?;

        Ok(rt)
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        self.scheduler.start(&self.bus)?;
        self.bus.spin().await
    }
}


⸻

5 Minimal agent example

// crates/logger-agent/src/lib.rs
use toka_core::{Agent, Event, EventKind};

pub struct Logger;
impl Agent for Logger {
    fn interest(&self) -> Vec<EventKind> { vec![EventKind::Any] }
    async fn handle(&self, ev: Event) -> anyhow::Result<()> {
        println!("{ev:?}");
        Ok(())
    }
}

Compile it statically into portable builds or drop it as a plugin .so.

⸻

6 Extension workflow
	1.	Add new heavy module (toka-ledger-core)
	•	zero external deps → safe to enable in portable build.
	2.	Add real DB backend (toka-ledger-pg)
	•	lives in its own crate; only activated via feature ledger-pg in runtime or via a cloud micro-service, keeping the portable binary small.
	3.	Write new driver (driver-notion)
	•	publishes to crates/driver-notion; runtime compiles it when feature enabled or loads .so.

⸻

Take-away
	•	toka-runtime remains lean: Tokio, serde, sled (optional), tracing.
	•	All heavier logic (vault, ledger, drivers, providers, DB back-ends) resides in separate crates but can be compiled into a single, self-contained binary via feature flags.
	•	Config TOML cleanly expresses only what the current build understands—unknown sections are ignored at runtime.

This pattern delivers ultra portability and clean modularity, giving you a clutter-free developer experience today while preserving the option to “bolt on” sophisticated services tomorrow.