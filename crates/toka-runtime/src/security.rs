//! Runtime security envelope – slice 5
//!
//! This module centralises **all** runtime-level authentication & redaction
//! concerns so that the rest of the runtime can stay agnostic of crypto
//! details.  It introduces two main building blocks:
//!
//! 1. [`SecretPool`] – lightweight container that holds one *active* secret
//!    plus a small ring-buffer of *retired* secrets still valid for a short
//!    grace period.  Rotation is O(1) and atomic.
//! 2. [`Envelope`] – thin wrapper that exposes a [`TokenValidator`] built
//!    from the active secret and a helper for redacting sensitive strings in
//!    logs.
//!
//! The design deliberately avoids any heavy crypto or persistence; callers
//! are expected to persist the *current* secret externally (e.g. Vault).
#![cfg(feature = "auth")]

use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use rand::{distributions::Alphanumeric, Rng};
use toka_capability::prelude::{JwtValidator, TokenValidator};
use tracing_subscriber::{fmt::MakeWriter, fmt, prelude::*};
use std::io::{self, Write};
use toka_capability::core::{Result, Error as CapError};

/// Maximum number of retired secrets kept alive for validation.
const MAX_OLD_SECRETS: usize = 4;

/// Generates a random 256-bit base-64 string suitable for HS256.
pub fn random_secret() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(43) // 43 chars ≈ 256 bits entropy (log2(62) * 43 ≈ 255.9)
        .map(char::from)
        .collect()
}

/// In-memory store for the active secret + recently rotated secrets.
#[derive(Debug)]
pub struct SecretPool {
    /// (secret, expiration) tuples; index 0 = *active* secret.
    secrets: Vec<(String, Instant)>,
    ttl: Duration,
}

impl SecretPool {
    /// Build a new pool with `initial` secret and a `ttl` for retired keys.
    pub fn new(initial: String, ttl: Duration) -> Self {
        Self {
            secrets: vec![(initial, Instant::now() + ttl)],
            ttl,
        }
    }

    /// Access the current active secret.
    pub fn current(&self) -> &str {
        &self.secrets[0].0
    }

    /// Rotate the active secret, retiring the previous one but keeping it
    /// valid for the configured `ttl` so in-flight requests succeed.
    pub fn rotate(&mut self) {
        // expire outdated secrets
        let now = Instant::now();
        self.secrets.retain(|(_, exp)| *exp > now);
        if self.secrets.len() >= MAX_OLD_SECRETS {
            self.secrets.pop(); // drop oldest
        }
        // push new active secret at front
        self.secrets.insert(0, (random_secret(), now + self.ttl));
    }

    /// Iterate over *all* currently valid secrets.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.secrets.iter().map(|(s, _)| s.as_str())
    }
}

/// Runtime-wide auth & redaction façade.
#[derive(Debug, Clone)]
pub struct Envelope {
    inner: Arc<RwLock<SecretPool>>,
}

impl Envelope {
    /// Create a new envelope with `initial` secret and given `ttl` for retired
    /// keys.
    pub fn initialise(initial: String, retired_ttl: Duration) -> Self {
        Self {
            inner: Arc::new(RwLock::new(SecretPool::new(initial, retired_ttl))),
        }
    }

    /// Produce a *validator* that accepts any of the valid secrets.
    pub fn validator(&self) -> MultiValidator {
        let pool = self.inner.read();
        let validators: Vec<JwtValidator> = pool
            .iter()
            .map(|s| JwtValidator::new(s))
            .collect();
        MultiValidator { validators }
    }

    /// Rotate secrets – typically invoked by a scheduler or admin API.
    pub fn rotate(&self) {
        self.inner.write().rotate();
    }

    /// Redact secrets found inside `input`, replacing them with `***`.
    pub fn redact(&self, input: &str) -> String {
        let pool = self.inner.read();
        let mut out = input.to_owned();
        for s in pool.iter() {
            out = out.replace(s, "***");
        }
        out
    }
}

/// Simple validator that tries a stack of [`JwtValidator`] instances.
#[derive(Clone)]
pub struct MultiValidator {
    validators: Vec<JwtValidator>,
}

#[async_trait::async_trait]
impl TokenValidator for MultiValidator {
    async fn validate(&self, raw: &str) -> Result<toka_capability::Claims> {
        for v in &self.validators {
            if let Ok(c) = v.validate(raw).await {
                return Ok(c);
            }
        }
        Err(CapError::new("token validation failed for all secrets"))
    }
}

// -------------------------------------------------------------------------------------------------
// Tracing redaction layer
// -------------------------------------------------------------------------------------------------

/// Writer that redacts secrets on the fly before delegating to stdout.
struct RedactingWriter {
    envelope: Envelope,
    inner: io::Stdout,
}

impl Write for RedactingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = String::from_utf8_lossy(buf);
        let redacted = self.envelope.redact(&s);
        self.inner.write(redacted.as_bytes())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// Factory for [`RedactingWriter`] so we can plug it into `tracing-subscriber`.
#[derive(Clone)]
struct RedactingMakeWriter {
    envelope: Envelope,
}

impl<'a> MakeWriter<'a> for RedactingMakeWriter {
    type Writer = RedactingWriter;

    fn make_writer(&'a self) -> Self::Writer {
        RedactingWriter {
            envelope: self.envelope.clone(),
            inner: io::stdout(),
        }
    }
}

/// Install a global tracing subscriber that automatically redacts capability
/// secrets in log output.
pub fn install_redacted_tracing(envelope: Envelope) {
    let fmt_layer = fmt::layer().with_writer(RedactingMakeWriter {
        envelope: envelope.clone(),
    });

    let _ = tracing_subscriber::registry()
        .with(fmt_layer)
        .try_init();
} 