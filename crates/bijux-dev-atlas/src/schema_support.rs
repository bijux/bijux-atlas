// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::Path;

use serde_json::Value;

#[allow(dead_code)]
pub(crate) fn read_json(path: &Path) -> Result<Value, String> {
    let text =
        fs::read_to_string(path).map_err(|err| format!("read {} failed: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("parse {} failed: {err}", path.display()))
}

pub(crate) fn require_object_keys(value: &Value, required: &[&str]) -> Result<(), Vec<String>> {
    let Some(object) = value.as_object() else {
        return Err(vec!["document root must be an object".to_string()]);
    };
    let errors = required
        .iter()
        .filter(|key| !object.contains_key(**key))
        .map(|key| format!("missing required key `{key}`"))
        .collect::<Vec<_>>();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_object_keys_accepts_matching_document() {
        let instance = serde_json::json!({"name": "atlas"});
        assert!(require_object_keys(&instance, &["name"]).is_ok());
    }

    #[test]
    fn require_object_keys_reports_missing_keys() {
        let instance = serde_json::json!({"name": "atlas"});
        let errors =
            require_object_keys(&instance, &["name", "version"]).expect_err("validation must fail");
        assert!(!errors.is_empty());
    }

    #[test]
    fn read_json_reads_document_from_disk() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("sample.json");
        fs::write(&path, "{\"name\":\"atlas\"}").expect("write sample");
        let value = read_json(&path).expect("read json");
        assert_eq!(value["name"].as_str(), Some("atlas"));
    }
}
