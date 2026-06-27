// SPDX-License-Identifier: Apache-2.0

use serde_json::{json, Value};

pub fn diagnose_bundle_payload(bundle_path: &str, files: Value) -> Value {
    json!({
        "schema_version": 1,
        "text": "ops diagnose bundle",
        "rows": [{
            "bundle": bundle_path,
            "files": files
        }],
        "summary": {"total": 1, "errors": 0, "warnings": 0}
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnose_bundle_payload_tracks_bundle_pointer() {
        let payload = diagnose_bundle_payload(
            "artifacts/ops/diagnose/atlas-run/bundle.json",
            json!(["artifacts/ops/scenarios/chaos/run-a/network.json"]),
        );

        assert_eq!(payload["text"], "ops diagnose bundle");
        assert_eq!(
            payload["rows"][0]["bundle"],
            "artifacts/ops/diagnose/atlas-run/bundle.json"
        );
        assert_eq!(payload["summary"]["errors"], 0);
    }
}
