// SPDX-License-Identifier: Apache-2.0

use crate::inventory::path_contracts::atlas_pins_manifest_from_repo_root;
use std::path::Path;

pub fn build_pins_check_payload(repo_root: &Path) -> Result<(serde_json::Value, i32), String> {
    let pins =
        crate::workspace::inventory::load_stack_pins(repo_root).map_err(|err| err.detail())?;
    let errors = crate::workspace::inventory::validate_pins_completeness(repo_root, &pins)
        .map_err(|err| err.detail())?;
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": status,
        "text": if errors.is_empty() { "ops pins check passed" } else { "ops pins check failed" },
        "rows": [{
            "pins_path": atlas_pins_manifest_from_repo_root(repo_root).display().to_string(),
            "errors": errors
        }],
        "summary": {"total": 1, "errors": if status == "ok" { 0 } else { 1 }, "warnings": 0}
    });
    Ok((payload, if status == "ok" { 0 } else { 1 }))
}

#[cfg(test)]
mod tests {
    use super::build_pins_check_payload;

    fn write_contract_root(root: &std::path::Path) {
        std::fs::create_dir_all(root.join("ops/inventory")).expect("mkdir inventory");
        std::fs::create_dir_all(root.join("ops/stack/generated")).expect("mkdir generated");
        std::fs::create_dir_all(root.join("ops/k8s/charts/bijux-atlas")).expect("mkdir chart");
        std::fs::write(
            root.join("ops/stack/generated/version-manifest.json"),
            "{\"schema_version\":1,\"redis\":\"redis@sha256:123\"}",
        )
        .expect("write version manifest");
        std::fs::write(
            root.join("ops/k8s/charts/bijux-atlas/values.yaml"),
            "image: redis@sha256:123\n",
        )
        .expect("write values");
        std::fs::write(
            root.join("ops/k8s/charts/bijux-atlas/values-offline.yaml"),
            "image: redis@sha256:123\n",
        )
        .expect("write offline values");
        std::fs::write(
            root.join("ops/inventory/contracts.json"),
            "{\"contracts\":[{\"path\":\"ops/inventory/tools.toml\"},{\"path\":\"ops/inventory/pins.yaml\"}]}",
        )
        .expect("write contracts");
    }

    #[test]
    fn pins_check_payload_reports_green_inventory_contract() {
        let root = tempfile::tempdir().expect("tempdir");
        write_contract_root(root.path());
        std::fs::write(
            root.path().join("ops/inventory/pins.yaml"),
            "images:\n  redis: \"redis@sha256:123\"\nversions:\n  chart: \"1.2.3\"\n  prometheus_operator_crd: \"0.78.2\"\n",
        )
        .expect("write pins");

        let (payload, exit_code) = build_pins_check_payload(root.path()).expect("payload");

        assert_eq!(exit_code, 0);
        assert_eq!(payload["status"], "ok");
        assert_eq!(payload["summary"]["errors"], 0);
    }

    #[test]
    fn pins_check_payload_reports_owned_errors() {
        let root = tempfile::tempdir().expect("tempdir");
        write_contract_root(root.path());
        std::fs::write(
            root.path().join("ops/inventory/pins.yaml"),
            "images:\n  redis: \"redis:latest\"\nversions:\n  chart: \"1.2.3\"\n  prometheus_operator_crd: \"0.78.2\"\n",
        )
        .expect("write pins");

        let (payload, exit_code) = build_pins_check_payload(root.path()).expect("payload");

        assert_eq!(exit_code, 1);
        assert_eq!(payload["status"], "failed");
        assert!(payload["rows"][0]["errors"]
            .as_array()
            .expect("errors")
            .iter()
            .any(|entry| entry
                .as_str()
                .is_some_and(|detail| detail.contains("floating tag forbidden"))));
    }
}
