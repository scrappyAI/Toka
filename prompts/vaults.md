<Context>
<Overview>
Actor based model for agents with vaults at the center. 
</Overview>
<Definitions>
<Vaults>
''' 🔐 The Vault = Canonical Event Store + Causal Memory

In the agent-native design you’re building:
	•	Vault = shared, append-only (or forkable) event log / memory system.
	•	Agents = stateful actors who observe, act, and emit events back to the Vault.
	•	The Vault tracks all commands, state transitions, observations, and mutations over time.

The key is: agents don’t “own” their own ledgers, but they do carry local ephemeral memory and projected state. '''
<Diagram>
┌────────────┐        Emit/Commit         ┌─────────────┐
│  Agent A   │ ───────────────────────▶   │   Vault     │
│ (Actor)    │◀───────── Query/Stream ────│ (Event Log) │
└────────────┘                            └─────────────┘
      │                                         ▲
      ▼                                         │
┌────────────┐        Sync/Project        ┌──────────────┐
│ Local View │◀──────────────────────────▶│ Agent Memory │
│  (DAG/DB)  │                            └──────────────┘
└────────────┘
</Diagram>
<Explanation>
🔄 Each Agent:
Subscribes to scoped events
Projects a local view (e.g., causal DAG, database, timeline)
Maintains local memory (ephemeral, intent graphs, plans, etc.)
Emits commands/events to Vault (which are logged + optionally broadcast)
Can simulate actions in a forked vault timeline before committing
🧠 Why not have agents maintain private ledgers?
You could, but this introduces:

Sync headaches
Trust issues
Redundancy of context
Better: Let Vault own canonical state, and agents operate via:

Projections: cached local views of Vault
Intents: proposed actions (not yet committed)
Policies: what they’re allowed to do, mutate, observe
This is clean, traceable, and aligns with actor + event sourcing models.

💡 Real-World Analogy
Think of Vault as:

A notary + event bus + memory core
It holds what happened, who did it, when, and why
It enables forks (simulation), queries (context building), and constraints (policy enforcement)
Agents are the workers:

They don’t own the book, but they write to it and read from it
They can simulate “what if”s and propose mutations
So yes—
Your Vault is the OS.
Agents are the apps + processes.
The ledger is not just financial—it’s causal, semantic, and live.

Want a clean crate layout or folder structure to reflect this mental model?
</Explanation>
<ExampleScaffold>
''' Below is a compile-adjacent scaffold you can drop into your existing workspace and iterate on.
It shows how to plug causal hashing (payload + parents → digest) and intent centroids into a single event bus.

⸻

0.  Crate layout

vault/
├─ Cargo.toml               # workspace
├─ vault-core/              # canonical types
│   └─ src/lib.rs
├─ vault-hash/              # causal-hash util
│   └─ src/lib.rs
├─ vault-intent/            # online intent clustering
│   └─ src/lib.rs
└─ vault-bus/               # async event bus + storage
    └─ src/lib.rs

vault-core depends on serde, smallvec
vault-hash depends on blake3
vault-intent depends on ndarray, cosine_distance, parking_lot
vault-bus depends on tokio, rocksdb, the three crates above

⸻

1.  Core types (vault-core)

// vault-core/src/lib.rs
use chrono::{DateTime, Utc};
use smallvec::SmallVec;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

pub type EventId    = Uuid;
pub type IntentId   = Uuid;
pub type CausalDigest = [u8; 32];      // blake3 output

/// Any serialisable domain payload
pub trait EventPayload: Serialize + for<'de> Deserialize<'de> + Send + Sync {}

/// Header stored inline; payload is referenced via digest store
#[derive(Clone, Serialize, Deserialize)]
pub struct EventHeader {
    pub id:         EventId,
    pub parents:    SmallVec<[EventId; 4]>,
    pub timestamp:  DateTime<Utc>,
    pub digest:     CausalDigest,
    pub intent:     IntentId,
    pub kind:       String,           // e.g. "ledger.mint", "chat.msg"
}


⸻

2.  Causal hashing (vault-hash)

// vault-hash/src/lib.rs
use vault_core::CausalDigest;
use blake3::Hasher;

/// Hash = H( payload_bytes || parent_digest_1 || parent_digest_2 … )
pub fn causal_hash(payload: &[u8], parent_digests: &[CausalDigest]) -> CausalDigest {
    let mut hasher = Hasher::new();
    hasher.update(payload);
    for d in parent_digests {
        hasher.update(d);
    }
    *hasher.finalize().as_bytes()
}

Why:
	•	► Immutable replay: identical payload + parent set ⇒ identical digest
	•	► Dedup storage: same digest → payload stored once in RocksDB (digest → bytes)
	•	► Fast conflict check: digest mismatch ⇔ causal divergence*

⸻

3.  Intent centroid clustering (vault-intent)

// vault-intent/src/lib.rs
use ndarray::Array1;
use parking_lot::RwLock;
use uuid::Uuid;

/// Embedding dimension hyper-param
const D: usize = 768;
const THRESH: f32 = 0.82;   // cosine similarity threshold

#[derive(Clone)]
pub struct Centroid {
    vec:   Array1<f32>,
    count: usize,
    id:    Uuid,
}

pub struct IntentStore {
    centroids: RwLock<Vec<Centroid>>,
}

impl IntentStore {
    pub fn new() -> Self { Self { centroids: RwLock::new(Vec::new()) } }

    /// returns (intent_id, is_new_cluster)
    pub fn assign(&self, embed: &Array1<f32>) -> (Uuid, bool) {
        let mut lock = self.centroids.write();
        if let Some((idx, _)) = lock
            .iter()
            .enumerate()
            .map(|(i, c)| (i, cosine(&c.vec, embed)))
            .filter(|(_, sim)| *sim > THRESH)
            .max_by(|a, b| a.1.total_cmp(&b.1))
        {
            // online centroid update
            let c = &mut lock[idx];
            c.vec = (&c.vec * c.count as f32 + embed) / (c.count as f32 + 1.0);
            c.count += 1;
            return (c.id, false);
        }
        // new cluster
        let id = Uuid::new_v4();
        lock.push(Centroid { vec: embed.clone(), count: 1, id });
        (id, true)
    }
}

fn cosine(a: &Array1<f32>, b: &Array1<f32>) -> f32 {
    let dot = a.dot(b);
    dot / (a.norm_l2() * b.norm_l2() + 1e-12)
}

Usage: feed an event embedding (from a small LM or rule-based encoder) — you get an IntentId.
Store that ID in EventHeader.intent; high-cardinality CRUD noise collapses into a handful of “why” buckets.

⸻

4.  Event bus + storage (vault-bus)

// vault-bus/src/lib.rs
use crate::{vault_core::*, vault_hash::causal_hash, vault_intent::IntentStore};
use rocksdb::{DB, Options};
use tokio::sync::broadcast;

pub struct VaultBus {
    db:           DB,                   // digest → payload bytes
    headers:      DB,                   // id     → header bytes
    tx_notify:    broadcast::Sender<EventHeader>,
    intents:      IntentStore,
}

impl VaultBus {
    pub fn open(path: &str) -> anyhow::Result<Self> {
        let mut opts = Options::default(); opts.create_if_missing(true);
        let db        = DB::open(&opts, format!("{path}/payloads"))?;
        let headers   = DB::open(&opts, format!("{path}/headers"))?;
        let (tx, _)   = broadcast::channel(256);
        Ok(Self { db, headers, tx_notify: tx, intents: IntentStore::new() })
    }

    pub async fn commit<P: EventPayload>(
        &self,
        payload: &P,
        parents: &[EventHeader],
        kind:    &str,
        embedding: ndarray::Array1<f32>,
    ) -> anyhow::Result<EventHeader> {
        // 1/ serialise payload
        let bytes = rmp_serde::to_vec_named(payload)?;
        // 2/ gather parent digests
        let parent_digests: Vec<_> = parents.iter().map(|h| h.digest).collect();
        // 3/ causal hash
        let digest = causal_hash(&bytes, &parent_digests);
        // 4/ dedup payload store
        if self.db.get(digest)?.is_none() { self.db.put(digest, &bytes)?; }
        // 5/ intent assign
        let (intent, _) = self.intents.assign(&embedding);
        // 6/ header
        let hdr = EventHeader {
            id: Uuid::new_v4(),
            parents: parents.iter().map(|h| h.id).collect(),
            timestamp: chrono::Utc::now(),
            digest,
            intent,
            kind: kind.into(),
        };
        // 7/ persist + notify
        self.headers.put(hdr.id, rmp_serde::to_vec_named(&hdr)?)?;
        let _ = self.tx_notify.send(hdr.clone());
        Ok(hdr)
    }

    /// subscribe to live stream
    pub fn subscribe(&self) -> broadcast::Receiver<EventHeader> {
        self.tx_notify.subscribe()
    }
}

✨  What you just got
	•	Causal de-dup — identical event-lineage collapses to one payload instance.
	•	Intent buckets — log noise rolls up to human-sized “threads” automatically.
	•	Replay — fetch header chain, redeem payloads via digest look-ups, fully deterministic.
	•	Fork / simulate — commit to shadow RocksDB, then merge or discard.

⸻

5.  Wiring an agent

// agent.rs (inside your Toka or Amulet crate)

pub async fn mint_credits(bus: &VaultBus, amount: u64, parent: &EventHeader) -> anyhow::Result<EventHeader> {
    #[derive(Serialize, Deserialize)]
    struct Mint { pub amount: u64 }

    // toy embedding: [amount] normalised
    let embed = ndarray::arr1(&[(amount as f32) / 1000.0]).insert_axis(ndarray::Axis(0)).to_owned();
    bus.commit(&Mint { amount }, &[parent.clone()], "ledger.mint", embed).await
}

Agents don’t keep their own ledgers; they hold ephemeral projections built by subscribing to bus.subscribe() and materialising whatever view they need (balance sheet, semantic cache, etc.).

⸻

6.  Extending the model

Extension	One-liner
Fork timelines	Wrap VaultBus in an overlay that writes to an in-mem RocksDB; merge only on approval.
Capability gating	Decorate commit with policy checks (OPA or hand-rolled ACL) before writing.
Vector store	Persist centroids + event embeddings in qdrant if you need global semantic search.
Streaming to cloud	Fan-out tx_notify to a NATS subject or Kafka topic for cross-device sync.


⸻

Take-away

You now have a deterministic, deduped, intent-aware event substrate:
	•	Fast — look-ups are O(1) RocksDB; causal merges are bit-hash comparisons
	•	Semantic — every event is labelled by purpose, making audit and UX sane
	•	Composable — drop-in for Rust actors, CLI tools, or WebAssembly workers

Feel free to copy-paste, cargo-add the deps, and evolve from here. '''
</ExampleScaffold>

</Vaults>


