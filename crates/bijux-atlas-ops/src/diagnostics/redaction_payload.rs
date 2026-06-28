// SPDX-License-Identifier: Apache-2.0

use super::path_contracts::diagnose_redacted_bundle_path;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub const DIAGNOSE_REDACTION_KEYS: [&str; 4] = ["password", "secret", "token", "api_key"];

pub fn redact_bundle_metadata(parsed: &mut Value) -> Vec<String> {
    let mut redacted = Vec::new();
    if let Some(object) = parsed.as_object_mut() {
        for key in DIAGNOSE_REDACTION_KEYS {
            if object.remove(key).is_some() {
                redacted.push(key.to_string());
            }
        }
        object.insert(
            "redaction_policy".to_string(),
            json!(DIAGNOSE_REDACTION_KEYS),
        );
        object.insert("redaction_applied".to_string(), json!(true));
    }
    redacted
}

pub fn write_redacted_bundle(
    bundle_path: &Path,
    parsed: &Value,
) -> Result<std::path::PathBuf, String> {
    let out_path = diagnose_redacted_bundle_path(bundle_path);
    fs::write(
        &out_path,
        serde_json::to_string_pretty(parsed)
            .map_err(|err| format!("failed to encode {}: {err}", out_path.display()))?,
    )
    .map_err(|err| format!("failed to write {}: {err}", out_path.display()))?;
    Ok(out_path)
}

pub fn diagnose_redaction_payload(
    source: &str,
    redacted: &str,
    redacted_keys: Vec<String>,
) -> Value {
    json!({
        "schema_version": 1,
        "text": "ops diagnose redact",
        "rows": [{
            "source": source,
            "redacted": redacted,
            "redacted_keys": redacted_keys,
            "policy_keys": DIAGNOSE_REDACTION_KEYS
        }],
        "summary": {"total": 1, "errors": 0, "warnings": 0}
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn redaction_removes_sensitive_metadata_keys() {
        let mut parsed = json!({
            "password": "secret",
            "token": "abc",
            "safe": true
        });

        let redacted = redact_bundle_metadata(&mut parsed);

        assert_eq!(redacted, vec!["password".to_string(), "token".to_string()]);
        assert!(parsed.get("password").is_none());
        assert_eq!(parsed["redaction_applied"], true);
    }

    #[test]
    fn redacted_bundle_writer_keeps_output_beside_the_source_bundle() {
        let repo_root = tempdir().expect("temp dir should exist");
        let bundle_path = repo_root.path().join("bundle.json");
        fs::write(&bundle_path, "{}").expect("seed bundle");

        let written = write_redacted_bundle(&bundle_path, &json!({"redaction_applied": true}))
            .expect("redacted bundle should write");

        assert_eq!(written, repo_root.path().join("bundle.redacted.json"));
    }
}
