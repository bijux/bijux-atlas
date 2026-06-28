// SPDX-License-Identifier: Apache-2.0

use sha2::{Digest, Sha256};
use std::path::Path;

const PINS_REL: &str = "ops/inventory/pins.yaml";
const STACK_REL: &str = "ops/stack/generated/version-manifest.json";
const TOOLCHAIN_REL: &str = "ops/inventory/toolchain.json";

pub fn build_pins_index_payload(
    repo_root: &Path,
    run_id: &str,
) -> Result<serde_json::Value, String> {
    let pins_raw = std::fs::read_to_string(repo_root.join(PINS_REL))
        .map_err(|err| format!("failed to read {PINS_REL}: {err}"))?;
    let toolchain_raw = std::fs::read_to_string(repo_root.join(TOOLCHAIN_REL))
        .map_err(|err| format!("failed to read {TOOLCHAIN_REL}: {err}"))?;
    let stack_raw = std::fs::read_to_string(repo_root.join(STACK_REL))
        .map_err(|err| format!("failed to read {STACK_REL}: {err}"))?;

    let mut files = vec![
        serde_json::json!({
            "path": PINS_REL,
            "sha256": sha256_hex(&pins_raw),
            "bytes": pins_raw.len()
        }),
        serde_json::json!({
            "path": STACK_REL,
            "sha256": sha256_hex(&stack_raw),
            "bytes": stack_raw.len()
        }),
        serde_json::json!({
            "path": TOOLCHAIN_REL,
            "sha256": sha256_hex(&toolchain_raw),
            "bytes": toolchain_raw.len()
        }),
    ];
    files.sort_by(|left, right| left["path"].as_str().cmp(&right["path"].as_str()));

    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": run_id,
        "generator": "ops generate pins-index",
        "files": files
    }))
}

fn sha256_hex(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::build_pins_index_payload;

    #[test]
    fn pins_index_payload_reads_owned_inventory_contracts() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/inventory")).expect("mkdir inventory");
        std::fs::create_dir_all(root.path().join("ops/stack/generated")).expect("mkdir stack");
        std::fs::write(
            root.path().join("ops/inventory/pins.yaml"),
            "images:\n  redis: \"redis@sha256:123\"\n",
        )
        .expect("write pins");
        std::fs::write(
            root.path().join("ops/inventory/toolchain.json"),
            r#"{"tools":{"helm":{"required":true}}}"#,
        )
        .expect("write toolchain");
        std::fs::write(
            root.path()
                .join("ops/stack/generated/version-manifest.json"),
            r#"{"schema_version":1,"redis":"redis@sha256:123"}"#,
        )
        .expect("write stack");

        let payload = build_pins_index_payload(root.path(), "owned-run").expect("pins index");
        let files = payload["files"].as_array().expect("files array");

        assert_eq!(payload["generator"], "ops generate pins-index");
        assert_eq!(files.len(), 3);
        assert_eq!(files[0]["path"], "ops/inventory/pins.yaml");
        assert_eq!(files[1]["path"], "ops/inventory/toolchain.json");
        assert_eq!(
            files[2]["path"],
            "ops/stack/generated/version-manifest.json"
        );
    }
}
