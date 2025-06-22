//! Smart Embedder – summarise structured events with an LLM and produce dense
//! embeddings suitable for clustering and search.
//!
//! The crate is intentionally minimal and pluggable:
//! * `Llm` trait abstracts any completion backend (OpenAI, local llama.cpp, etc.)
//! * `SentenceEncoder` trait abstracts sentence → `Array1<f32>` encoding.
//! * `SmartEmbedder` ties both together.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use anyhow::Result;
use async_trait::async_trait;
use ndarray::Array1;
use serde::Serialize;

/// Abstract Large Language Model client.
#[async_trait]
pub trait Llm: Send + Sync {
    /// Complete the given prompt and return the generated text.
    async fn complete(&self, prompt: &str) -> Result<String>;
}

/// Encode a single sentence into a dense vector.
#[async_trait]
pub trait SentenceEncoder: Send + Sync {
    /// Embed the sentence into an `Array1<f32>` of fixed dimension.
    async fn encode(&self, sentence: &str) -> Result<Array1<f32>>;
}

/// Smart embedder that converts structured payloads into semantic embeddings.
pub struct SmartEmbedder<L, E> {
    llm:      L,
    encoder:  E,
}

impl<L, E> SmartEmbedder<L, E>
where
    L: Llm + 'static,
    E: SentenceEncoder + 'static,
{
    /// Create a new instance.
    pub fn new(llm: L, encoder: E) -> Self {
        Self { llm, encoder }
    }

    /// Summarise `payload` of kind `kind` into an embedding vector.
    pub async fn embed_event<P: Serialize>(&self, kind: &str, payload: &P) -> Result<Array1<f32>> {
        // 1. Serialise payload as pretty JSON for the prompt.
        let json = serde_json::to_string_pretty(payload)?;
        let prompt = format!(
            r#"You are an event summariser agent. Summarise the following {kind} event in one short sentence:

Event Payload (JSON):
{json}

Summary:"#
        );

        // 2. Get summary from LLM.
        let summary = self.llm.complete(&prompt).await?;

        // 3. Encode summary.
        self.encoder.encode(&summary).await
    }
}

// ------------------------------------------------------------
// Simple reference implementations behind features
// ------------------------------------------------------------

#[cfg(feature = "openai")]
mod openai_client {
    use super::*;
    use reqwest::Client;

    /// Basic OpenAI-compatible chat client.
    pub struct OpenAiClient {
        pub api_key: String,
        pub base_url: String,
        http:        Client,
    }

    impl OpenAiClient {
        /// Create a new client. `base_url` can be "https://api.openai.com".
        pub fn new(api_key: String, base_url: String) -> Self {
            Self {
                api_key,
                base_url,
                http: Client::new(),
            }
        }
    }

    #[async_trait]
    impl Llm for OpenAiClient {
        async fn complete(&self, prompt: &str) -> Result<String> {
            #[derive(serde::Serialize)]
            struct Req<'a> {
                model: &'a str,
                messages: [Message<'a>; 2],
                temperature: f32,
            }
            #[derive(serde::Serialize)]
            struct Message<'a> {
                role: &'a str,
                content: &'a str,
            }
            let req = Req {
                model: "gpt-3.5-turbo",
                messages: [
                    Message { role: "system", content: "You are an event summarizer." },
                    Message { role: "user", content: prompt },
                ],
                temperature: 0.2,
            };

            let resp: serde_json::Value = self
                .http
                .post(format!("{}/v1/chat/completions", self.base_url))
                .bearer_auth(&self.api_key)
                .json(&req)
                .send()
                .await?
                .json()
                .await?;

            Ok(
                resp["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("")
                    .trim()
                    .to_string(),
            )
        }
    }
}

#[cfg(feature = "transformers")]
mod transformer_encoder {
    use super::*;
    use rust_bert::pipelines::sentence_embeddings::SentenceEmbeddingsModel;

    /// Wrapper around `rust-bert` sentence embedding model.
    pub struct BertEncoder {
        model: SentenceEmbeddingsModel,
    }

    impl BertEncoder {
        /// Load the default `all-MiniLM-L6-v2` model.
        pub fn new() -> anyhow::Result<Self> {
            let model = SentenceEmbeddingsModel::new(Default::default())?;
            Ok(Self { model })
        }
    }

    #[async_trait]
    impl SentenceEncoder for BertEncoder {
        async fn encode(&self, sentence: &str) -> Result<Array1<f32>> {
            let vecs = self.model.encode(&[sentence])?;
            Ok(vecs[0].clone())
        }
    }
} 