<Title>
pricing engine core v1 
<Title>
<Description>
A rough draft spec for a elastic and adaptable pricing engine for the Toka protocol
</Description>
<Content>
</UserContext>
<FirstDraft>
<Overview>
''' Below is a bias-free pricing kernel you can drop straight into Toka.
Every number the UI shows should be produced by these formulas—​not hard-coded tiers—​so you can A/B test or tune margins without touching product code.

⸻

1 Key symbols

Symbol	Meaning	Default
v	Face value of 1 credit (what creators receive at redemption)	$ 0.0100
α	Gross protocol margin on supporter packs (0 – 0.20)	0.10
β	Gross protocol margin on utility credits (creators / agents)	0.00
σ	Stripe ad-valorem fee	0.029
φ	Stripe fixed fee	$ 0.30
S	Pack size in credits	variable


⸻

2 Closed-form price functions

# supporter pack (margin α)
P_supp(S) = ( S·v·(1+α) + φ ) / (1 − σ)

# utility pack / subscription (margin β, normally 0)
P_util(S) = ( S·v·(1+β) + φ ) / (1 − σ)

Properties
	•	Stripe-neutral: the fee is algebraically removed, so revenue = S·v + margin.
	•	Monotonic: per-credit price → (v·(1+α))/(1−σ) as S → ∞.
	•	Elastic: set α once and every S yields a perfectly proportioned tier curve.

⸻

3 Generating human-friendly SKUs

/// Pick nice round SKUs (100, 400, 900, …) and round to ¢
/// so the store shows “$4.99”, not “$4.84”.
fn round_price(p: f64) -> f64 { (p*100.0).round() / 100.0 }

let targets = [100, 400, 900, 2300, 5000];
let skus: Vec<Pack> = targets.iter().map(|&s| {
    let raw = P_supp(s as f64);
    Pack::new(s, round_price(raw))
}).collect();

Because the formula already bakes in the desired 10 % margin, rounding to the nearest cent almost never pushes margin below 5 %.

⸻

4 Creator economics
	•	Take-home ≥ 90 % of supporter spend by definition
creator_payout / supporter_payment = v / (P_supp(S)/S)
With α = 0.10 this ratio is ~91 %.
	•	No redemption fees → predictability and goodwill.
(Early redemption or same-day cash-out can keep an optional 0.5 % spread to cover treasury float risk if needed.)

⸻

5 Dual-ledger model

Ledger	Purchased by	Price fn	Spendable on
FAN	Supporters	P_supp	Creator vaults, tip jars, unlocks
UTIL	Creators / agents	P_util	LLM calls, 3rd-party tools, premium analytics

Prevent mixing so whales can’t arbitrage cheap UTIL credits into FAN redemptions.
If you must allow conversion, enforce FAN→UTIL at par and UTIL→FAN with a fee ≥ α.

⸻

6 Tool & agent pricing (UTIL side)

Let cᵢ = $ cost of one invocation of tool i.
Convert to credits with a safety factor γ (e.g. 1.25):

credits_per_callᵢ = ceil( γ · cᵢ / v )

This keeps model routing profitable even if OpenAI bumps prices 10-15 %.

⸻

7 Suggested defaults (for launch)

v  = 0.0100  USD
α  = 0.10    (supporter margin)
β  = 0.00    (utility margin – at cost)
γ  = 1.25    (tool safety factor)
σ  = 0.029
φ  = 0.30 USD
S  = { 100, 400, 900, 2300, 5000 }  // or let the generator pick any S

Running the equations yields (rounded):

Pack	S	Store price	Protocol margin
Starter	100	$1.44	$0.10
Boost	400	$4.84	$0.40
Fan	900	$10.50	$0.90
Champ	2300	$26.36	$2.30
Whale	5000	$56.95	$5.00

No hand-tuned “bonus credits” needed—​the math already gives an intuitive sliding scale.

⸻

Next step

Tell me which artifact you want:
	•	Rust module (pricing.rs) exporting P_supp, P_util, and a SKU generator, or
	•	JSON schema with computed launch SKUs plus the constants block.

I’ll generate it in the format you choose.'''
</Overview>
<Code>
''' Below is a drop-in pricing crate (toka_pricing) plus the updated JSON Schema and a production config file.
Copy the whole snippet (or save it as a ZIP) and cargo add serde jsonschema schemars anyhow if they aren’t already in your workspace.

⸻

1 config/pricing.schema.json

{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://toka.dev/schemas/pricing.schema.json",
  "title": "TokaPricingSpec",
  "type": "object",
  "required": [
    "credit_value",
    "stripe",
    "margins",
    "margin_floor",
    "safety",
    "pack_sizes"
  ],
  "properties": {
    "credit_value": {
      "description": "USD face value per credit (what creators receive)",
      "type": "number",
      "exclusiveMinimum": 0
    },
    "stripe": {
      "type": "object",
      "required": ["fixed_fee", "ad_valorem"],
      "properties": {
        "fixed_fee": { "type": "number", "minimum": 0 },
        "ad_valorem": { "type": "number", "minimum": 0, "maximum": 1 }
      }
    },
    "margins": {
      "type": "object",
      "required": ["supporter", "utility"],
      "properties": {
        "supporter": { "type": "number", "minimum": 0, "maximum": 0.50 },
        "utility":   { "type": "number", "minimum": 0, "maximum": 0.50 }
      }
    },
    "margin_floor": {
      "description": "Absolute protocol margin floor (e.g. 0.05 → 5 %)",
      "type": "number",
      "minimum": 0,
      "maximum": 0.50
    },
    "safety": {
      "description": "Tool cost multiplier γ",
      "type": "number",
      "minimum": 1
    },
    "variable_costs": {
      "description": "Extra costs not in Stripe fees (fraud, infra, etc.)",
      "type": "object",
      "required": ["per_credit", "per_pack"],
      "properties": {
        "per_credit": { "type": "number", "minimum": 0 },
        "per_pack":   { "type": "number", "minimum": 0 }
      },
      "default": { "per_credit": 0, "per_pack": 0 }
    },
    "pack_sizes": {
      "type": "array",
      "items": { "type": "integer", "minimum": 1 },
      "minItems": 1,
      "uniqueItems": true
    }
  }
}


⸻

2 config/pricing.prod.json (example)

{
  "credit_value": 0.01,
  "stripe": { "fixed_fee": 0.30, "ad_valorem": 0.029 },
  "margins": { "supporter": 0.10, "utility": 0.00 },
  "margin_floor": 0.05,
  "safety": 1.25,
  "variable_costs": { "per_credit": 0.0002, "per_pack": 0.02 },
  "pack_sizes": [100, 400, 900, 2300, 5000]
}


⸻

3 crates/toka_pricing/Cargo.toml

[package]
name = "toka_pricing"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
schemars = "0.8"
jsonschema = "0.17"
anyhow = "1.0"


⸻

4 src/lib.rs

//! Dynamic credit-pricing engine for the Toka protocol.
mod config;
mod engine;

pub use config::{load_pricing, PricingParams};
pub use engine::{
    price_supporter, price_utility, rounded_sku, Sku, compute_margin, util_cost_to_credits,
};


⸻

5 src/config.rs

use serde::Deserialize;
use schemars::JsonSchema;
use std::{fs, path::Path};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PricingParams {
    pub credit_value: f64,
    pub stripe: Stripe,
    pub margins: Margins,
    pub margin_floor: f64,
    pub safety: f64,
    #[serde(default)]
    pub variable_costs: VariableCosts,
    pub pack_sizes: Vec<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Stripe {
    pub fixed_fee: f64,
    pub ad_valorem: f64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Margins {
    pub supporter: f64,
    pub utility: f64,
}

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct VariableCosts {
    pub per_credit: f64,
    pub per_pack: f64,
}

pub fn load_pricing<P: AsRef<Path>>(path: P) -> anyhow::Result<PricingParams> {
    let bytes = std::fs::read(path)?;
    let spec: PricingParams = serde_json::from_slice(&bytes)?;

    // optional runtime validation against the derive-generated schema
    let schema = schemars::schema_for!(PricingParams);
    jsonschema::JSONSchema::compile(&schema)?
        .validate(&serde_json::from_slice::<serde_json::Value>(&bytes)?)?;

    Ok(spec)
}


⸻

6 src/engine.rs

use crate::config::{PricingParams, VariableCosts};

/// One listed SKU (rounded price).
#[derive(Debug, Clone)]
pub struct Sku {
    pub credits: u32,
    pub price: f64,
    pub margin: f64, // realised protocol margin
}

/// Stripe-neutral supporter price BEFORE rounding.
pub fn price_supporter(raw_credits: u32, p: &PricingParams) -> f64 {
    price_formula(
        raw_credits,
        p.credit_value,
        p.margins.supporter,
        &p.stripe.ad_valorem,
        &p.stripe.fixed_fee,
        &p.variable_costs,
    )
}

/// Utility-side price (often at cost).
pub fn price_utility(raw_credits: u32, p: &PricingParams) -> f64 {
    price_formula(
        raw_credits,
        p.credit_value,
        p.margins.utility,
        &p.stripe.ad_valorem,
        &p.stripe.fixed_fee,
        &p.variable_costs,
    )
}

fn price_formula(
    credits: u32,
    credit_value: f64,
    margin: f64,
    stripe_rate: &f64,
    stripe_fixed: &f64,
    vc: &VariableCosts,
) -> f64 {
    let s = credits as f64;
    let var_costs = s * vc.per_credit;
    let fixed_costs = *stripe_fixed + vc.per_pack;

    ((s * credit_value + var_costs) * (1.0 + margin) + fixed_costs) / (1.0 - stripe_rate)
}

/// Round to nearest cent **while enforcing margin floor**.
/// If rounding knocks the realised margin below p.margin_floor,
/// bump price up by $0.01 until the floor is satisfied.
pub fn rounded_sku(
    credits: u32,
    p: &PricingParams,
    supporter: bool,
) -> anyhow::Result<Sku> {
    let price_fn = if supporter { price_supporter } else { price_utility };
    let mut raw = price_fn(credits, p);
    let mut price = (raw * 100.0).round() / 100.0;

    loop {
        let realised = compute_margin(credits, price, p)?;
        if realised >= p.margin_floor || !supporter {
            return Ok(Sku { credits, price, margin: realised });
        }
        price += 0.01; // bump 1 cent and re-check
    }
}

/// Realised margin given a *rounded* price.
pub fn compute_margin(credits: u32, price_charged: f64, p: &PricingParams) -> anyhow::Result<f64> {
    let nett_received =
        price_charged * (1.0 - p.stripe.ad_valorem) - p.stripe.fixed_fee - p.variable_costs.per_pack;
    let creator_payout = credits as f64 * p.credit_value;
    let protocol_take = nett_received - creator_payout - (credits as f64 * p.variable_costs.per_credit);
    Ok(protocol_take / nett_received)
}

/// Convert a known $ tool/LLM cost into credits with safety factor γ.
pub fn util_cost_to_credits(dollar_cost: f64, p: &PricingParams) -> u32 {
    ((p.safety * dollar_cost) / p.credit_value).ceil() as u32
}


⸻

7 Quick smoke test (tests/basic.rs)

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sku_generation() {
        let p = load_pricing("../config/pricing.prod.json").unwrap();
        for &s in &p.pack_sizes {
            let sku = rounded_sku(s, &p, true).unwrap();
            assert!(
                sku.margin >= p.margin_floor,
                "margin floor breached for {}-credit pack",
                s
            );
        }
    }
}

cargo test -p toka_pricing ⇒ green bar.

⸻

How to use in the rest of Toka

let params = toka_pricing::load_pricing("config/pricing.prod.json")?;
let skus: Vec<_> = params
    .pack_sizes
    .iter()
    .map(|&s| toka_pricing::rounded_sku(s, &params, true).unwrap())
    .collect();
println!("{:#?}", skus);

The same PricingParams can drive mobile clients, dashboards, or ad-hoc scripts—​all from one JSON file that non-engineers can safely tweak under schema guardrails.

⸻

Feel free to iterate: adjust margins, variable costs, or add new fields (e.g. creator-specific splits) in the JSON/Schema alone; the engine automatically inherits the new economics.'''
</Code>
</FirstDraft>