// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

pub fn inventory_contract_errors(ops_root: &Path) -> Vec<String> {
    match crate::inventory::ops_inventory::OpsInventory::load_and_validate(ops_root) {
        Ok(_) => Vec::new(),
        Err(error) => vec![error],
    }
}

pub fn collect_advisory_inventory_errors(repo_root: &Path, ops_root: &Path) -> Vec<String> {
    let mut errors = inventory_contract_errors(ops_root);
    if let Ok(pins) = crate::workspace::inventory::load_stack_pins(repo_root) {
        if let Ok(pin_errors) =
            crate::workspace::inventory::validate_pins_completeness(repo_root, &pins)
        {
            errors.extend(pin_errors);
        }
    }
    if let Ok(stack_manifest) = crate::workspace::stack::load_stack_manifest(repo_root) {
        errors.extend(crate::workspace::stack::validate_stack_manifest(
            repo_root,
            &stack_manifest,
        ));
    }
    if let Ok(load_manifest) = crate::workspace::load::load_load_manifest(repo_root) {
        errors.extend(crate::workspace::load::validate_load_manifest(
            repo_root,
            &load_manifest,
        ));
    }
    errors.sort();
    errors.dedup();
    errors
}

pub fn collect_strict_inventory_errors(
    repo_root: &Path,
    ops_root: &Path,
) -> Result<Vec<String>, String> {
    let mut errors = inventory_contract_errors(ops_root);
    let pins =
        crate::workspace::inventory::load_stack_pins(repo_root).map_err(|err| err.detail())?;
    errors.extend(
        crate::workspace::inventory::validate_pins_completeness(repo_root, &pins)
            .map_err(|err| err.detail())?,
    );
    let stack_manifest =
        crate::workspace::stack::load_stack_manifest(repo_root).map_err(|err| err.detail())?;
    errors.extend(crate::workspace::stack::validate_stack_manifest(
        repo_root,
        &stack_manifest,
    ));
    let load_manifest =
        crate::workspace::load::load_load_manifest(repo_root).map_err(|err| err.detail())?;
    errors.extend(crate::workspace::load::validate_load_manifest(
        repo_root,
        &load_manifest,
    ));
    errors.sort();
    errors.dedup();
    Ok(errors)
}

pub fn inventory_summary_or_error(repo_root: &Path) -> serde_json::Value {
    crate::inventory::ops_inventory::ops_inventory_summary(repo_root)
        .unwrap_or_else(|err| serde_json::json!({"error": format!("OPS_MANIFEST_ERROR: {err}")}))
}

#[cfg(test)]
mod tests {
    use super::{
        collect_advisory_inventory_errors, collect_strict_inventory_errors,
        inventory_contract_errors, inventory_summary_or_error,
    };

    fn write_inventory_fixture(root: &std::path::Path) -> std::path::PathBuf {
        let ops_root = root.join("ops");
        std::fs::create_dir_all(ops_root.join("stack/generated")).expect("mkdir generated");
        std::fs::create_dir_all(ops_root.join("inventory")).expect("mkdir inventory");
        std::fs::create_dir_all(ops_root.join("k8s/charts/bijux-atlas")).expect("mkdir chart");
        std::fs::create_dir_all(ops_root.join("load/k6/suites")).expect("mkdir load suites");
        std::fs::create_dir_all(ops_root.join("load/queries")).expect("mkdir load queries");
        std::fs::create_dir_all(ops_root.join("load/thresholds")).expect("mkdir thresholds");
        std::fs::write(
            ops_root.join("stack/profiles.json"),
            r#"{"schema_version":1,"profiles":[{"name":"kind","kind_profile":"atlas-kind","cluster_config":"ops/kind/kind.yaml"}]}"#,
        )
        .expect("write profiles");
        std::fs::write(
            ops_root.join("stack/generated/version-manifest.json"),
            r#"{"schema_version":1,"redis":"redis@sha256:123"}"#,
        )
        .expect("write version manifest");
        std::fs::write(
            ops_root.join("inventory/toolchain.json"),
            r#"{"schema_version":1,"images":{"redis":"redis@sha256:123"},"tools":{"cargo":{"required":true,"version_regex":"1\\..*","probe_argv":["cargo","--version"]}}}"#,
        )
        .expect("write toolchain");
        std::fs::write(
            ops_root.join("inventory/surfaces.json"),
            r#"{"schema_version":2,"actions":[{"id":"check","domain":"ops","command":["bijux","dev","atlas","check","run"]}]}"#,
        )
        .expect("write surfaces");
        std::fs::write(
            ops_root.join("inventory/generated-committed-mirror.json"),
            r#"{"schema_version":1,"mirrors":[]}"#,
        )
        .expect("write mirror");
        std::fs::write(
            ops_root.join("inventory/contracts.json"),
            r#"{"schema_version":1}"#,
        )
        .expect("write contracts");
        std::fs::write(
            ops_root.join("inventory/gates.json"),
            r#"{"schema_version":1,"gates":[]}"#,
        )
        .expect("write gates");
        std::fs::write(
            ops_root.join("inventory/pins.yaml"),
            "images:\n  redis: \"redis@sha256:123\"\nversions:\n  chart: \"1.2.3\"\n  prometheus_operator_crd: \"0.78.2\"\n",
        )
        .expect("write pins");
        std::fs::write(
            ops_root.join("stack/stack.toml"),
            "[profiles.kind]\nkind_profile=\"atlas-kind\"\ncluster_config=\"ops/kind/kind.yaml\"\nnamespace=\"bijux-atlas\"\ncomponents=[]\n",
        )
        .expect("write stack manifest");
        std::fs::write(ops_root.join("k8s/charts/bijux-atlas/values.yaml"), "").expect("values");
        std::fs::write(
            ops_root.join("k8s/charts/bijux-atlas/values-offline.yaml"),
            "",
        )
        .expect("values offline");
        std::fs::write(
            ops_root.join("load/load.toml"),
            "[suites.smoke]\nscript=\"ops/load/k6/suites/mixed-80-20.js\"\ndataset=\"ops/load/queries/pinned-v1.json\"\nthresholds=\"ops/load/thresholds/mixed.thresholds.json\"\n",
        )
        .expect("write load manifest");
        std::fs::write(ops_root.join("load/k6/suites/mixed-80-20.js"), "").expect("script");
        std::fs::write(ops_root.join("load/queries/pinned-v1.json"), "{}").expect("query");
        std::fs::write(ops_root.join("load/thresholds/mixed.thresholds.json"), "{}")
            .expect("thresholds");
        ops_root
    }

    #[test]
    fn contract_errors_surface_primary_inventory_failures() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops")).expect("mkdir ops");

        let errors = inventory_contract_errors(&root.path().join("ops"));

        assert!(!errors.is_empty());
    }

    #[test]
    fn advisory_errors_ignore_absent_secondary_manifests() {
        let root = tempfile::tempdir().expect("tempdir");
        let ops_root = write_inventory_fixture(root.path());
        std::fs::remove_file(root.path().join("ops/load/load.toml")).expect("remove load");

        let errors = collect_advisory_inventory_errors(root.path(), &ops_root);

        assert!(errors.iter().all(|entry| !entry.contains("load suite")));
    }

    #[test]
    fn strict_errors_require_owned_manifests() {
        let root = tempfile::tempdir().expect("tempdir");
        let ops_root = write_inventory_fixture(root.path());
        std::fs::remove_file(root.path().join("ops/load/load.toml")).expect("remove load");

        let error = collect_strict_inventory_errors(root.path(), &ops_root).expect_err("strict");

        assert!(error.contains("ops/load/load.toml"));
    }

    #[test]
    fn inventory_summary_uses_owned_inventory_surface() {
        let root = tempfile::tempdir().expect("tempdir");
        let _ops_root = write_inventory_fixture(root.path());

        let summary = inventory_summary_or_error(root.path());

        assert!(summary.get("stack_profiles").is_some());
        assert!(summary.get("surface_actions").is_some());
    }
}
