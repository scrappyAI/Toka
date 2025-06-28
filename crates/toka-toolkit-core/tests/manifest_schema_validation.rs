use toka_toolkit_core::manifest::{ToolManifest, Schema, Transport};
use serde_json::json;
use anyhow::Result;

fn base_manifest() -> ToolManifest {
    ToolManifest {
        id: "demo.tool".into(),
        name: "Demo Tool".into(),
        version: "0.1.0".into(),
        description: "A demo tool".into(),
        capability: "demo".into(),
        side_effect: Default::default(),
        input_schema: None,
        output_schema: None,
        transports: vec![Transport::InProcess],
        action_id: None,
        manifest_version: toka_toolkit_core::manifest::SCHEMA_VERSION.to_string(),
        protocols: vec![],
        metadata: Default::default(),
    }
}

#[test]
fn valid_schema_passes() -> Result<()> {
    let mut mani = base_manifest();
    let input = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "required": ["text"],
        "properties": { "text": { "type": "string", "minLength": 1 } }
    });
    mani.input_schema = Some(Schema(input.to_string()));

    mani.validate()?;
    Ok(())
}

#[test]
fn syntax_error_fails() {
    let mut mani = base_manifest();
    mani.input_schema = Some(Schema("{ not valid json".into()));
    assert!(mani.validate().is_err());
}

#[cfg(not(feature = "allow_remote_refs"))]
#[test]
fn remote_ref_disallowed() {
    let mut mani = base_manifest();
    let schema = json!({ "$ref": "https://example.com/schema.json" });
    mani.input_schema = Some(Schema(schema.to_string()));
    assert!(mani.validate().is_err());
}

#[cfg(feature = "allow_remote_refs")]
#[test]
fn remote_ref_allowed_feature_enabled() -> Result<()> {
    let mut mani = base_manifest();
    let schema = json!({ "$ref": "https://example.com/schema.json" });
    mani.input_schema = Some(Schema(schema.to_string()));
    // Should validate successfully when the feature is enabled
    mani.validate()?;
    Ok(())
}

#[test]
fn oversize_schema_fails() {
    let mut mani = base_manifest();
    let big_prop = "a".repeat(70_000);
    let schema = json!({ "type": "string", "x": big_prop });
    mani.input_schema = Some(Schema(schema.to_string()));
    assert!(mani.validate().is_err());
}