fn validate_dataset_fixture_policies_and_filesystem(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let manifest_rel = Path::new("ops/datasets/manifest.json");
    let manifest_dataset_ids = if ctx.adapters.fs.exists(ctx.repo_root, manifest_rel) {
        let manifest_text = fs::read_to_string(ctx.repo_root.join(manifest_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let manifest_json: serde_json::Value = serde_json::from_str(&manifest_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        manifest_json
            .get("datasets")
            .and_then(|v| v.as_array())
            .map(|datasets| {
                datasets
                    .iter()
                    .filter_map(|entry| entry.get("id").and_then(|v| v.as_str()))
                    .map(ToString::to_string)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default()
    } else {
        BTreeSet::new()
    };

    let consumer_list_rel = Path::new("ops/datasets/consumer-list.json");
    if ctx.adapters.fs.exists(ctx.repo_root, consumer_list_rel) {
        let consumer_list_text = fs::read_to_string(ctx.repo_root.join(consumer_list_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let consumer_list_json: serde_json::Value = serde_json::from_str(&consumer_list_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if let Some(consumers) = consumer_list_json.get("consumers").and_then(|v| v.as_array()) {
            for consumer in consumers {
                let consumer_id = consumer
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown-consumer");
                for dataset_id in consumer
                    .get("dataset_ids")
                    .and_then(|v| v.as_array())
                    .into_iter()
                    .flatten()
                    .filter_map(|v| v.as_str())
                {
                    if !manifest_dataset_ids.contains(dataset_id) {
                        violations.push(violation(
                            "OPS_DATASET_CONSUMER_UNKNOWN_DATASET_ID",
                            format!(
                                "dataset consumer `{consumer_id}` references unknown dataset id `{dataset_id}`"
                            ),
                            "align ops/datasets/consumer-list.json dataset_ids with ops/datasets/manifest.json",
                            Some(consumer_list_rel),
                        ));
                    }
                }
            }
        }
    } else {
        violations.push(violation(
            "OPS_DATASET_CONSUMER_LIST_MISSING",
            format!("missing dataset consumer contract `{}`", consumer_list_rel.display()),
            "restore ops/datasets/consumer-list.json",
            Some(consumer_list_rel),
        ));
    }

    let freeze_policy_rel = Path::new("ops/datasets/freeze-policy.json");
    if ctx.adapters.fs.exists(ctx.repo_root, freeze_policy_rel) {
        let freeze_policy_text = fs::read_to_string(ctx.repo_root.join(freeze_policy_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let freeze_policy_json: serde_json::Value = serde_json::from_str(&freeze_policy_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        for key in [
            "schema_version",
            "freeze_mode",
            "immutability",
            "retention",
            "change_controls",
        ] {
            if freeze_policy_json.get(key).is_none() {
                violations.push(violation(
                    "OPS_DATASET_FREEZE_POLICY_FIELD_MISSING",
                    format!("freeze policy missing required key `{key}`"),
                    "add missing required keys to ops/datasets/freeze-policy.json",
                    Some(freeze_policy_rel),
                ));
            }
        }
    } else {
        violations.push(violation(
            "OPS_DATASET_FREEZE_POLICY_MISSING",
            format!("missing dataset freeze policy `{}`", freeze_policy_rel.display()),
            "restore ops/datasets/freeze-policy.json",
            Some(freeze_policy_rel),
        ));
    }

    let fixture_policy_rel = Path::new("ops/datasets/fixture-policy.json");
    let mut allowed_binary_paths = BTreeSet::new();
    if ctx.adapters.fs.exists(ctx.repo_root, fixture_policy_rel) {
        let policy_text = fs::read_to_string(ctx.repo_root.join(fixture_policy_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let policy_json: serde_json::Value = serde_json::from_str(&policy_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        for key in [
            "schema_version",
            "allow_remote_download",
            "fixture_roots",
            "allowed_kinds",
            "allowed_binary_paths",
            "policy",
        ] {
            if policy_json.get(key).is_none() {
                violations.push(violation(
                    "OPS_FIXTURE_POLICY_FIELD_MISSING",
                    format!("fixture policy missing required key `{key}`"),
                    "add missing required fixture policy key",
                    Some(fixture_policy_rel),
                ));
            }
        }
        let configured = policy_json
            .get("allowed_binary_paths")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(ToString::to_string)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        allowed_binary_paths.extend(configured);
    } else {
        violations.push(violation(
            "OPS_FIXTURE_POLICY_MISSING",
            format!(
                "missing fixture policy file `{}`",
                fixture_policy_rel.display()
            ),
            "restore ops/datasets/fixture-policy.json",
            Some(fixture_policy_rel),
        ));
    }

    let fixtures_root = ctx.repo_root.join("ops/datasets/fixtures");
    if fixtures_root.exists() {
        let allowed_root_docs = BTreeSet::from([
            "ops/datasets/fixtures/README.md".to_string(),
            "ops/datasets/fixtures/CONTRACT.md".to_string(),
            "ops/datasets/fixtures/INDEX.md".to_string(),
            "ops/datasets/fixtures/OWNER.md".to_string(),
        ]);
        for file in walk_files(&fixtures_root) {
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
            let rel_str = rel.display().to_string();
            if allowed_root_docs.contains(&rel_str) {
                continue;
            }
            if rel_str.contains("/assets/")
                && rel_str.contains("/v")
                && !rel_str.ends_with(".tar.gz")
                && rel_str.starts_with("ops/datasets/fixtures/")
            {
                violations.push(violation(
                    "OPS_FIXTURE_VERSION_ASSET_TARBALL_REQUIRED",
                    format!(
                        "fixture version assets must be .tar.gz archives: `{}`",
                        rel.display()
                    ),
                    "keep version asset payloads under assets/ with .tar.gz extension",
                    Some(rel),
                ));
            }
            if is_binary_like_file(&file)?
                && !rel_str.ends_with(".tar.gz")
                && !allowed_binary_paths.contains(&rel_str)
            {
                violations.push(violation(
                    "OPS_FIXTURE_BINARY_POLICY_VIOLATION",
                    format!(
                        "binary fixture file is not allowlisted and not a fixture tarball: `{}`",
                        rel.display()
                    ),
                    "allowlist the binary in fixture-policy.json or replace with a tarball fixture asset",
                    Some(rel),
                ));
            }
        }

        for entry in
            fs::read_dir(&fixtures_root).map_err(|err| CheckError::Failed(err.to_string()))?
        {
            let entry = entry.map_err(|err| CheckError::Failed(err.to_string()))?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
                continue;
            };
            if name == "." || name == ".." {
                continue;
            }
            let mut has_version_dir = false;
            for child in fs::read_dir(&path).map_err(|err| CheckError::Failed(err.to_string()))? {
                let child = child.map_err(|err| CheckError::Failed(err.to_string()))?;
                let child_path = child.path();
                let Some(child_name) = child_path.file_name().and_then(|v| v.to_str()) else {
                    continue;
                };
                if child_path.is_dir() && child_name.starts_with('v') {
                    has_version_dir = true;
                } else if child_path.is_file() {
                    let rel = child_path
                        .strip_prefix(ctx.repo_root)
                        .unwrap_or(child_path.as_path());
                    violations.push(violation(
                        "OPS_FIXTURE_LOOSE_FILE_FORBIDDEN",
                        format!(
                            "fixture family `{name}` has loose file outside versioned subtree: `{}`",
                            rel.display()
                        ),
                        "place fixture files under versioned directories like v1/",
                        Some(rel),
                    ));
                }
            }
            if !has_version_dir {
                let rel = path.strip_prefix(ctx.repo_root).unwrap_or(path.as_path());
                violations.push(violation(
                    "OPS_FIXTURE_VERSION_DIRECTORY_MISSING",
                    format!(
                        "fixture family `{name}` must contain versioned directories (v1, v2, ...)"
                    ),
                    "create versioned fixture subdirectory and move fixture payloads into it",
                    Some(rel),
                ));
            }
        }

        for manifest in walk_files(&fixtures_root)
            .into_iter()
            .filter(|p| p.file_name().and_then(|v| v.to_str()) == Some("manifest.lock"))
        {
            let manifest_rel = manifest
                .strip_prefix(ctx.repo_root)
                .unwrap_or(manifest.as_path());
            let content =
                fs::read_to_string(&manifest).map_err(|err| CheckError::Failed(err.to_string()))?;
            let mut archive_name = None::<String>;
            let mut sha256 = None::<String>;
            for line in content.lines() {
                if let Some(v) = line.strip_prefix("archive=") {
                    archive_name = Some(v.trim().to_string());
                }
                if let Some(v) = line.strip_prefix("sha256=") {
                    sha256 = Some(v.trim().to_string());
                }
            }
            let Some(archive_name) = archive_name else {
                violations.push(violation(
                    "OPS_FIXTURE_MANIFEST_ARCHIVE_MISSING",
                    format!(
                        "manifest lock missing archive= entry: `{}`",
                        manifest_rel.display()
                    ),
                    "add archive=<filename> to fixture manifest.lock",
                    Some(manifest_rel),
                ));
                continue;
            };
            let Some(expected_sha) = sha256 else {
                violations.push(violation(
                    "OPS_FIXTURE_MANIFEST_SHA_MISSING",
                    format!(
                        "manifest lock missing sha256= entry: `{}`",
                        manifest_rel.display()
                    ),
                    "add sha256=<digest> to fixture manifest.lock",
                    Some(manifest_rel),
                ));
                continue;
            };
            let version_dir = manifest
                .parent()
                .ok_or_else(|| CheckError::Failed("manifest.lock parent not found".to_string()))?;
            let tarball_path = version_dir.join("assets").join(&archive_name);
            let tarball_rel = tarball_path
                .strip_prefix(ctx.repo_root)
                .unwrap_or(tarball_path.as_path());
            if !tarball_path.exists() {
                violations.push(violation(
                    "OPS_FIXTURE_TARBALL_MISSING",
                    format!(
                        "fixture tarball declared by manifest.lock is missing: `{}`",
                        tarball_rel.display()
                    ),
                    "restore tarball under versioned assets/ directory",
                    Some(manifest_rel),
                ));
                continue;
            }
            let actual_sha = sha256_hex(&tarball_path)?;
            if actual_sha != expected_sha {
                violations.push(violation(
                    "OPS_FIXTURE_TARBALL_HASH_MISMATCH",
                    format!(
                        "fixture tarball hash mismatch for `{}`: expected={} actual={}",
                        tarball_rel.display(),
                        expected_sha,
                        actual_sha
                    ),
                    "refresh manifest.lock sha256 after tarball update",
                    Some(manifest_rel),
                ));
            }

            let src_dir = version_dir.join("src");
            if !src_dir.exists() || !src_dir.is_dir() {
                violations.push(violation(
                    "OPS_FIXTURE_SRC_DIRECTORY_MISSING",
                    format!(
                        "fixture version missing src/ directory: `{}`",
                        src_dir
                            .strip_prefix(ctx.repo_root)
                            .unwrap_or(src_dir.as_path())
                            .display()
                    ),
                    "add src/ copies for fixture version inputs",
                    Some(manifest_rel),
                ));
            }
            let has_queries = walk_files(version_dir).iter().any(|p| {
                p.file_name()
                    .and_then(|v| v.to_str())
                    .is_some_and(|n| n.contains("queries"))
            });
            let has_responses = walk_files(version_dir).iter().any(|p| {
                p.file_name()
                    .and_then(|v| v.to_str())
                    .is_some_and(|n| n.contains("responses"))
            });
            if !has_queries || !has_responses {
                violations.push(violation(
                    "OPS_FIXTURE_GOLDENS_MISSING",
                    format!(
                        "fixture version must include query/response goldens: `{}`",
                        version_dir
                            .strip_prefix(ctx.repo_root)
                            .unwrap_or(version_dir)
                            .display()
                    ),
                    "add *queries*.json and *responses*.json goldens in fixture version",
                    Some(manifest_rel),
                ));
            }
        }
    }

    Ok(())
}

