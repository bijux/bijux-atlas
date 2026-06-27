// SPDX-License-Identifier: Apache-2.0

use serde_json::{json, Value};

pub fn diagnose_explain_payload(
    bundle_path: &str,
    kind: &str,
    run_id: &str,
    file_count: usize,
) -> Value {
    json!({
        "schema_version": 1,
        "text": "ops diagnose explain",
        "rows": [{
            "bundle": bundle_path,
            "kind": kind,
            "run_id": run_id,
            "file_count": file_count,
            "summary": if file_count == 0 { "no evidence files discovered" } else { "bundle contains evidence files" }
        }],
        "summary": {"total": 1, "errors": 0, "warnings": 0}
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnose_explain_payload_reflects_empty_bundles() {
        let payload = diagnose_explain_payload(
            "artifacts/ops/diagnose/atlas-run/bundle.json",
            "ops_diagnose_bundle",
            "atlas-run",
            0,
        );
        assert_eq!(payload["text"], "ops diagnose explain");
        assert_eq!(payload["rows"][0]["file_count"], 0);
        assert_eq!(
            payload["rows"][0]["summary"],
            "no evidence files discovered"
        );
    }
}
