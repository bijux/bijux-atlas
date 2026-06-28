fn validate_pins_file_content(
    repo_root: &Path,
    toolchain_image_keys: BTreeSet<String>,
    stack_component_keys: BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    let path = repo_root.join(OPS_PINS_PATH);
    let Ok(text) = fs::read_to_string(&path) else {
        return;
    };
    let parsed: PinsManifest = match serde_yaml::from_str(&text) {
        Ok(value) => value,
        Err(err) => {
            errors.push(format!("{OPS_PINS_PATH}: invalid yaml: {err}"));
            return;
        }
    };
    if parsed.schema_version != EXPECTED_PINS_SCHEMA {
        errors.push(format!(
            "{OPS_PINS_PATH}: expected schema_version={EXPECTED_PINS_SCHEMA}, got {}",
            parsed.schema_version
        ));
    }
    if parsed.images.is_empty() {
        errors.push(format!("{OPS_PINS_PATH}: images must not be empty"));
    }
    if parsed.dataset_ids.is_empty() {
        errors.push(format!("{OPS_PINS_PATH}: dataset_ids must not be empty"));
    }
    for (name, image) in &parsed.images {
        if image.contains(":latest") {
            errors.push(format!(
                "{OPS_PINS_PATH}: image `{name}` uses forbidden latest tag"
            ));
        }
        validate_image_hash(name, image, errors);
    }

    for required in toolchain_image_keys.union(&stack_component_keys) {
        if !parsed.images.contains_key(required) {
            errors.push(format!(
                "{OPS_PINS_PATH}: missing image pin `{required}` required by toolchain/stack manifests"
            ));
        }
    }
    for key in parsed.images.keys() {
        if !toolchain_image_keys.contains(key) || !stack_component_keys.contains(key) {
            errors.push(format!(
                "{OPS_PINS_PATH}: unused image pin `{key}` not present in both toolchain and stack manifests"
            ));
        }
    }

    let mut seen_dataset_ids = BTreeSet::new();
    for id in &parsed.dataset_ids {
        if id.trim().is_empty() {
            errors.push(format!(
                "{OPS_PINS_PATH}: dataset_ids must not contain empty entries"
            ));
            continue;
        }
        if !seen_dataset_ids.insert(id.clone()) {
            errors.push(format!("{OPS_PINS_PATH}: duplicate dataset pin `{id}`"));
        }
    }

    let datasets_path = repo_root.join(OPS_DATASETS_MANIFEST_PATH);
    if let Ok(dataset_text) = fs::read_to_string(&datasets_path) {
        match serde_json::from_str::<DatasetsManifest>(&dataset_text) {
            Ok(manifest) => {
                if manifest.schema_version < 1 {
                    errors.push(format!(
                        "{OPS_DATASETS_MANIFEST_PATH}: schema_version must be >= 1"
                    ));
                }
                let known_ids = manifest
                    .datasets
                    .iter()
                    .map(|entry| entry.id.clone())
                    .collect::<BTreeSet<_>>();
                for known in &known_ids {
                    if !seen_dataset_ids.contains(known) {
                        errors.push(format!(
                            "{OPS_PINS_PATH}: missing dataset pin `{known}` from {OPS_DATASETS_MANIFEST_PATH}"
                        ));
                    }
                }
                for pinned in &seen_dataset_ids {
                    if !known_ids.contains(pinned) {
                        errors.push(format!(
                            "{OPS_PINS_PATH}: unused dataset pin `{pinned}` not present in {OPS_DATASETS_MANIFEST_PATH}"
                        ));
                    }
                }
            }
            Err(err) => errors.push(format!(
                "{OPS_DATASETS_MANIFEST_PATH}: invalid json for dataset pin validation: {err}"
            )),
        }
    }

    for (name, version) in &parsed.versions {
        if !is_semver(version) {
            errors.push(format!(
                "{OPS_PINS_PATH}: version `{name}` must be semver (x.y.z), got `{version}`"
            ));
        }
    }
}

fn load_pins_manifest(repo_root: &Path) -> Result<PinsManifest, String> {
    let path = repo_root.join(OPS_PINS_PATH);
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_yaml::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn validate_image_hash(name: &str, image: &str, errors: &mut Vec<String>) {
    let Some(at_pos) = image.find('@') else {
        return;
    };
    let digest = &image[at_pos + 1..];
    if !digest.starts_with("sha256:") {
        errors.push(format!(
            "{OPS_PINS_PATH}: image `{name}` uses unsupported digest format (expected sha256)"
        ));
        return;
    }
    let raw = &digest["sha256:".len()..];
    if raw.len() != 64 || !raw.chars().all(|c| c.is_ascii_hexdigit()) {
        errors.push(format!(
            "{OPS_PINS_PATH}: image `{name}` has invalid sha256 digest length/content"
        ));
    }
}

fn is_semver(value: &str) -> bool {
    let mut parts = value.split('.');
    let (Some(major), Some(minor), Some(patch), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    [major, minor, patch]
        .iter()
        .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

