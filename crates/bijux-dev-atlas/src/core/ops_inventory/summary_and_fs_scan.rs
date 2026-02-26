pub fn ops_inventory_summary(repo_root: &Path) -> Result<serde_json::Value, String> {
    let inventory = load_ops_inventory_cached(repo_root)?;
    Ok(serde_json::json!({
        "stack_profiles": inventory.stack_profiles.profiles.len(),
        "surface_actions": inventory.surfaces.actions.len(),
        "toolchain_images": inventory.toolchain.images.len(),
        "mirror_entries": inventory.mirror_policy.mirrors.len(),
            "schema_versions": {
                "stack_profiles": inventory.stack_profiles.schema_version,
                "stack_version_manifest": inventory.stack_version_manifest.schema_version,
                "toolchain": inventory.toolchain.schema_version,
                "surfaces": inventory.surfaces.schema_version,
                "mirror_policy": inventory.mirror_policy.schema_version,
            "contracts": inventory.contracts.schema_version
        }
    }))
}

fn collect_files_recursive(path: PathBuf) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path];
    }
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            out.extend(collect_files_recursive(entry.path()));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{load_ops_inventory_cached, validate_pins_file_content};
    use std::collections::BTreeSet;
    use std::fs;

    #[test]
    fn pins_file_forbids_latest_and_invalid_digest_formats() {
        let root = tempfile::tempdir().expect("tempdir");
        let ops_inventory = root.path().join("ops/inventory");
        fs::create_dir_all(&ops_inventory).expect("mkdir");
        fs::create_dir_all(root.path().join("ops/datasets")).expect("mkdir datasets");
        fs::write(
            ops_inventory.join("pins.yaml"),
            "schema_version: 1\nimages:\n  app: repo/app:latest\n  good: repo/app:v1@sha256:abc\n  bad: repo/app:v1@sha1:abc\ndataset_ids:\n  - 110/homo_sapiens/GRCh38\nversions:\n  chart: 0.1.0\n",
        )
        .expect("write pins");
        fs::write(
            root.path().join("ops/datasets/manifest.json"),
            r#"{"schema_version":1,"datasets":[{"id":"110/homo_sapiens/GRCh38"}]}"#,
        )
        .expect("write datasets");
        let mut errors = Vec::new();
        validate_pins_file_content(
            root.path(),
            BTreeSet::from(["app".to_string(), "good".to_string(), "bad".to_string()]),
            BTreeSet::from(["app".to_string(), "good".to_string(), "bad".to_string()]),
            &mut errors,
        );
        let text = errors.join("\n");
        assert!(text.contains("forbidden latest tag"), "{text}");
        assert!(text.contains("unsupported digest format"), "{text}");
        assert!(text.contains("invalid sha256 digest"), "{text}");
    }

    #[test]
    fn cached_inventory_reload_detects_content_changes() {
        let root = tempfile::tempdir().expect("tempdir");
        let repo = root.path();
        fs::create_dir_all(repo.join("ops/stack")).expect("mkdir");
        fs::create_dir_all(repo.join("ops/inventory")).expect("mkdir");
        fs::write(
            repo.join("ops/stack/profiles.json"),
            r#"{"schema_version":1,"profiles":[{"name":"dev","kind_profile":"kind","cluster_config":"ops/kind/dev.yaml"}]}"#,
        )
        .expect("write profiles");
        fs::create_dir_all(repo.join("ops/stack/generated")).expect("mkdir generated");
        fs::write(
            repo.join("ops/stack/generated/version-manifest.json"),
            r#"{"schema_version":1,"rust":"ghcr.io/x/rust:1"}"#,
        )
        .expect("write version manifest");
        fs::write(
            repo.join("ops/inventory/toolchain.json"),
            r#"{"schema_version":1,"images":{"rust":"ghcr.io/x/rust:1"},"tools":{"cargo":{"required":true,"version_regex":"1\\..*","probe_argv":["cargo","--version"]}}}"#,
        )
        .expect("write toolchain");
        fs::write(repo.join("ops/inventory/pins.yaml"), "images: {}\n").expect("write pins");
        fs::write(
            repo.join("ops/inventory/surfaces.json"),
            r#"{"schema_version":2,"actions":[{"id":"check","domain":"ops","command":["bijux","dev","atlas","check","run"]}]}"#,
        )
        .expect("write surfaces");
        fs::write(
            repo.join("ops/inventory/generated-committed-mirror.json"),
            r#"{"schema_version":1,"mirrors":[]}"#,
        )
        .expect("write mirror");
        fs::write(
            repo.join("ops/inventory/contracts.json"),
            r#"{"schema_version":1}"#,
        )
        .expect("write contracts");
        fs::write(
            repo.join("ops/inventory/gates.json"),
            r#"{"schema_version":1,"gates":[]}"#,
        )
        .expect("write gates");

        let first = load_ops_inventory_cached(repo).expect("first");
        assert_eq!(
            first.toolchain.images.get("rust"),
            Some(&"ghcr.io/x/rust:1".to_string())
        );

        fs::write(
            repo.join("ops/inventory/toolchain.json"),
            r#"{"schema_version":1,"images":{"rust":"ghcr.io/x/rust:2"},"tools":{"cargo":{"required":true,"version_regex":"1\\..*","probe_argv":["cargo","--version"]}}}"#,
        )
        .expect("rewrite toolchain");

        let second = load_ops_inventory_cached(repo).expect("second");
        assert_eq!(
            second.toolchain.images.get("rust"),
            Some(&"ghcr.io/x/rust:2".to_string())
        );
    }

    #[test]
    fn pins_file_flags_missing_and_unused_pins() {
        let root = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(root.path().join("ops/inventory")).expect("mkdir inventory");
        fs::create_dir_all(root.path().join("ops/datasets")).expect("mkdir datasets");
        fs::write(
            root.path().join("ops/inventory/pins.yaml"),
            "schema_version: 1\nimages:\n  redis: redis:7.4-alpine\n  orphan: ghcr.io/example/orphan:1.0.0\ndataset_ids:\n  - 111/homo_sapiens/GRCh38\nversions:\n  chart: not-semver\n",
        )
        .expect("write pins");
        fs::write(
            root.path().join("ops/datasets/manifest.json"),
            r#"{"schema_version":1,"datasets":[{"id":"110/homo_sapiens/GRCh38"}]}"#,
        )
        .expect("write datasets");

        let mut errors = Vec::new();
        validate_pins_file_content(
            root.path(),
            BTreeSet::from(["redis".to_string()]),
            BTreeSet::from(["redis".to_string()]),
            &mut errors,
        );

        let text = errors.join("\n");
        assert!(text.contains("unused image pin `orphan`"), "{text}");
        assert!(
            text.contains("missing dataset pin `110/homo_sapiens/GRCh38`"),
            "{text}"
        );
        assert!(
            text.contains("unused dataset pin `111/homo_sapiens/GRCh38`"),
            "{text}"
        );
        assert!(text.contains("must be semver"), "{text}");
    }
}
