pub(super) fn check_ops_fixture_governance(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
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

    let e2e_fixture_root = ctx.repo_root.join("ops/e2e/fixtures");
    let allowlist_rel = Path::new("ops/e2e/fixtures/allowlist.json");
    let lock_rel = Path::new("ops/e2e/fixtures/fixtures.lock");
    if e2e_fixture_root.exists() && ctx.adapters.fs.exists(ctx.repo_root, allowlist_rel) {
        let allowlist_text = fs::read_to_string(ctx.repo_root.join(allowlist_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let allowlist_json: serde_json::Value = serde_json::from_str(&allowlist_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let allowed_paths = allowlist_json
            .get("allowed_paths")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|i| i.as_str())
                    .map(ToString::to_string)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        let actual_fixture_paths = walk_files(&e2e_fixture_root)
            .into_iter()
            .filter_map(|p| {
                p.strip_prefix(ctx.repo_root)
                    .ok()
                    .map(|r| r.display().to_string())
            })
            .collect::<BTreeSet<_>>();
        for path in &actual_fixture_paths {
            if !allowed_paths.contains(path) {
                violations.push(violation(
                    "OPS_E2E_FIXTURE_ALLOWLIST_VIOLATION",
                    format!("e2e fixture file not allowlisted: `{path}`"),
                    "add fixture path to ops/e2e/fixtures/allowlist.json",
                    Some(allowlist_rel),
                ));
            }
        }
        for path in &allowed_paths {
            if !actual_fixture_paths.contains(path) {
                violations.push(violation(
                    "OPS_E2E_FIXTURE_ALLOWLIST_STALE_ENTRY",
                    format!("allowlist references missing e2e fixture file: `{path}`"),
                    "remove stale path from allowlist or restore file",
                    Some(allowlist_rel),
                ));
            }
        }

        if ctx.adapters.fs.exists(ctx.repo_root, lock_rel) {
            let lock_text = fs::read_to_string(ctx.repo_root.join(lock_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let lock_json: serde_json::Value = serde_json::from_str(&lock_text)
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let expected = lock_json
                .get("allowlist_sha256")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let actual = sha256_hex(&ctx.repo_root.join(allowlist_rel))?;
            if expected != actual {
                violations.push(violation(
                    "OPS_E2E_FIXTURE_LOCK_DRIFT",
                    "fixtures.lock allowlist_sha256 does not match allowlist.json".to_string(),
                    "update fixtures.lock allowlist_sha256 when allowlist changes",
                    Some(lock_rel),
                ));
            }
            let expected_inventory_sha = lock_json
                .get("fixture_inventory_sha256")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let fixture_inventory_rel = Path::new("ops/datasets/generated/fixture-inventory.json");
            if ctx.adapters.fs.exists(ctx.repo_root, fixture_inventory_rel) {
                let actual_inventory_sha = sha256_hex(&ctx.repo_root.join(fixture_inventory_rel))?;
                if expected_inventory_sha != actual_inventory_sha {
                    violations.push(violation(
                        "OPS_E2E_FIXTURE_INVENTORY_LOCK_DRIFT",
                        "fixtures.lock fixture_inventory_sha256 does not match fixture-inventory.json"
                            .to_string(),
                        "update fixtures.lock fixture_inventory_sha256 when fixture inventory changes",
                        Some(lock_rel),
                    ));
                }
            }
        }
    }

    let suites_rel = Path::new("ops/e2e/suites/suites.json");
    let scenarios_rel = Path::new("ops/e2e/scenarios/scenarios.json");
    let expectations_rel = Path::new("ops/e2e/expectations/expectations.json");
    let scenario_slo_map_rel = Path::new("ops/inventory/scenario-slo-map.json");
    let observe_coverage_rel = Path::new("ops/inventory/observability-coverage-map.json");
    let alerts_contract_rel = Path::new("ops/observe/contracts/alerts-contract.json");
    let mut canonical_e2e_scenario_ids = BTreeSet::new();
    if ctx.adapters.fs.exists(ctx.repo_root, suites_rel) {
        let suites_text = fs::read_to_string(ctx.repo_root.join(suites_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let suites_json: serde_json::Value = serde_json::from_str(&suites_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let suite_ids = suites_json
            .get("suites")
            .and_then(|v| v.as_array())
            .map(|suites| {
                suites
                    .iter()
                    .filter_map(|suite| suite.get("id").and_then(|v| v.as_str()))
                    .map(ToString::to_string)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        let scenario_ids = if ctx.adapters.fs.exists(ctx.repo_root, scenarios_rel) {
            let scenarios_text = fs::read_to_string(ctx.repo_root.join(scenarios_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let scenarios_json: serde_json::Value = serde_json::from_str(&scenarios_text)
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            scenarios_json
                .get("scenarios")
                .and_then(|v| v.as_array())
                .map(|scenarios| {
                    scenarios
                        .iter()
                        .filter_map(|entry| entry.get("id").and_then(|v| v.as_str()))
                        .map(ToString::to_string)
                        .collect::<BTreeSet<_>>()
                })
                .unwrap_or_default()
        } else {
            BTreeSet::new()
        };
        canonical_e2e_scenario_ids.extend(scenario_ids.iter().cloned());
        if ctx.adapters.fs.exists(ctx.repo_root, expectations_rel) {
            let expectations_text = fs::read_to_string(ctx.repo_root.join(expectations_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let expectations_json: serde_json::Value = serde_json::from_str(&expectations_text)
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            for scenario_id in expectations_json
                .get("expectations")
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
                .filter_map(|entry| entry.get("scenario_id").and_then(|v| v.as_str()))
            {
                if !suite_ids.contains(scenario_id) && !scenario_ids.contains(scenario_id) {
                    violations.push(violation(
                        "OPS_E2E_EXPECTATION_REFERENCE_MISSING",
                        format!(
                            "expectations.json scenario_id `{scenario_id}` is missing from suites/scenarios registries"
                        ),
                        "align expectations entries with canonical suite ids or scenario ids",
                        Some(expectations_rel),
                    ));
                }
            }
        }
        if let Some(suites) = suites_json.get("suites").and_then(|v| v.as_array()) {
            for suite in suites {
                let Some(id) = suite.get("id").and_then(|v| v.as_str()) else {
                    continue;
                };
                let maybe_fixture = if id.starts_with("fixture-") {
                    id.strip_prefix("fixture-")
                } else if id.ends_with("-fixture") {
                    id.strip_suffix("-fixture")
                } else {
                    None
                };
                if let Some(name) = maybe_fixture {
                    let fixture_dir = Path::new("ops/datasets/fixtures").join(name);
                    if !ctx.adapters.fs.exists(ctx.repo_root, &fixture_dir) {
                        violations.push(violation(
                            "OPS_E2E_FIXTURE_REFERENCE_MISSING",
                            format!(
                                "e2e suite `{id}` references missing fixture family `{}`",
                                fixture_dir.display()
                            ),
                            "create fixture family directory or rename e2e suite id",
                            Some(suites_rel),
                        ));
                    }
                }
            }
        }
    }

    let smoke_manifest_rel = Path::new("ops/e2e/manifests/smoke.manifest.json");
    if ctx.adapters.fs.exists(ctx.repo_root, smoke_manifest_rel) {
        let smoke_manifest_text = fs::read_to_string(ctx.repo_root.join(smoke_manifest_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let smoke_manifest_json: serde_json::Value = serde_json::from_str(&smoke_manifest_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let queries_lock_rel = Path::new("ops/e2e/smoke/queries.lock");
        if smoke_manifest_json
            .get("queries_lock")
            .and_then(|v| v.as_str())
            .is_none()
        {
            violations.push(violation(
                "OPS_E2E_SMOKE_MANIFEST_PINNED_QUERIES_MISSING",
                "smoke.manifest.json must define `queries_lock`".to_string(),
                "reference the pinned query lock file from smoke.manifest.json",
                Some(smoke_manifest_rel),
            ));
        }
        if let Some(queries_lock_ref) = smoke_manifest_json
            .get("queries_lock")
            .and_then(|v| v.as_str())
        {
            if queries_lock_ref != queries_lock_rel.display().to_string() {
                violations.push(violation(
                    "OPS_E2E_SMOKE_MANIFEST_PINNED_QUERIES_PATH_INVALID",
                    format!(
                        "smoke manifest queries_lock must be `{}`; found `{queries_lock_ref}`",
                        queries_lock_rel.display()
                    ),
                    "point smoke manifest queries_lock to ops/e2e/smoke/queries.lock",
                    Some(smoke_manifest_rel),
                ));
            }
            if !ctx.adapters.fs.exists(ctx.repo_root, queries_lock_rel) {
                violations.push(violation(
                    "OPS_E2E_SMOKE_QUERIES_LOCK_MISSING",
                    format!("missing pinned query lock file `{}`", queries_lock_rel.display()),
                    "restore ops/e2e/smoke/queries.lock",
                    Some(smoke_manifest_rel),
                ));
            }
        }
    }

    let load_suites_rel = Path::new("ops/load/suites/suites.json");
    let mut load_suite_names = BTreeSet::new();
    if ctx.adapters.fs.exists(ctx.repo_root, load_suites_rel) {
        let load_suites_text = fs::read_to_string(ctx.repo_root.join(load_suites_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let load_suites_json: serde_json::Value = serde_json::from_str(&load_suites_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        for suite in load_suites_json
            .get("suites")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            let suite_name = suite
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            load_suite_names.insert(suite_name.to_string());
            if suite.get("kind").and_then(|v| v.as_str()) == Some("k6") {
                if let Some(scenario_name) = suite.get("scenario").and_then(|v| v.as_str()) {
                    let scenario_rel = Path::new("ops/load/scenarios").join(scenario_name);
                    if !ctx.adapters.fs.exists(ctx.repo_root, &scenario_rel) {
                        violations.push(violation(
                            "OPS_LOAD_SUITE_SCENARIO_MISSING",
                            format!(
                                "load suite `{suite_name}` references missing scenario `{}`",
                                scenario_rel.display()
                            ),
                            "add missing scenario file or fix suite scenario reference",
                            Some(load_suites_rel),
                        ));
                    }
                }
            }
            if let Some(threshold_file) = suite.get("threshold_file").and_then(|v| v.as_str()) {
                let threshold_rel = Path::new(threshold_file);
                if !ctx.adapters.fs.exists(ctx.repo_root, threshold_rel) {
                    violations.push(violation(
                        "OPS_LOAD_SUITE_THRESHOLD_FILE_MISSING",
                        format!(
                            "load suite `{suite_name}` references missing threshold file `{threshold_file}`"
                        ),
                        "add threshold file or remove stale threshold_file reference",
                        Some(load_suites_rel),
                    ));
                }
            }
        }
    }

    let slo_definitions_rel = Path::new("ops/observe/slo-definitions.json");
    let mut slo_ids = BTreeSet::new();
    if ctx.adapters.fs.exists(ctx.repo_root, slo_definitions_rel) {
        let slo_text = fs::read_to_string(ctx.repo_root.join(slo_definitions_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let slo_json: serde_json::Value =
            serde_json::from_str(&slo_text).map_err(|err| CheckError::Failed(err.to_string()))?;
        slo_ids = slo_json
            .get("slos")
            .and_then(|v| v.as_array())
            .map(|slos| {
                slos.iter()
                    .filter_map(|entry| entry.get("id").and_then(|v| v.as_str()))
                    .map(ToString::to_string)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
    }

    let drills_rel = Path::new("ops/inventory/drills.json");
    let mut drill_ids = BTreeSet::new();
    if ctx.adapters.fs.exists(ctx.repo_root, drills_rel) {
        let drills_text = fs::read_to_string(ctx.repo_root.join(drills_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let drills_json: serde_json::Value = serde_json::from_str(&drills_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        drill_ids = drills_json
            .get("drills")
            .and_then(|v| v.as_array())
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(ToString::to_string)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
    }

    if ctx.adapters.fs.exists(ctx.repo_root, scenario_slo_map_rel) {
        let map_text = fs::read_to_string(ctx.repo_root.join(scenario_slo_map_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let map_json: serde_json::Value =
            serde_json::from_str(&map_text).map_err(|err| CheckError::Failed(err.to_string()))?;
        let map_entries = map_json
            .get("mappings")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let mut mapped_scenarios = BTreeSet::new();
        for entry in &map_entries {
            let scenario_id = entry
                .get("scenario_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            if scenario_id.is_empty() {
                continue;
            }
            mapped_scenarios.insert(scenario_id.clone());

            for slo_id in entry
                .get("slo_ids")
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str())
            {
                if !slo_ids.contains(slo_id) {
                    violations.push(violation(
                        "OPS_SCENARIO_SLO_MAP_UNKNOWN_SLO_ID",
                        format!(
                            "scenario-slo-map entry `{scenario_id}` references unknown slo id `{slo_id}`"
                        ),
                        "align scenario-slo-map slo_ids with ops/observe/slo-definitions.json",
                        Some(scenario_slo_map_rel),
                    ));
                }
            }

            for drill_id in entry
                .get("drill_ids")
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str())
            {
                if !drill_ids.contains(drill_id) {
                    violations.push(violation(
                        "OPS_SCENARIO_SLO_MAP_UNKNOWN_DRILL_ID",
                        format!(
                            "scenario-slo-map entry `{scenario_id}` references unknown drill id `{drill_id}`"
                        ),
                        "align scenario-slo-map drill_ids with ops/inventory/drills.json",
                        Some(scenario_slo_map_rel),
                    ));
                }
            }

            for load_suite in entry
                .get("load_suites")
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str())
            {
                if !load_suite_names.contains(load_suite) {
                    violations.push(violation(
                        "OPS_SCENARIO_SLO_MAP_UNKNOWN_LOAD_SUITE",
                        format!(
                            "scenario-slo-map entry `{scenario_id}` references unknown load suite `{load_suite}`"
                        ),
                        "align scenario-slo-map load_suites with ops/load/suites/suites.json",
                        Some(scenario_slo_map_rel),
                    ));
                }
            }
        }

        for scenario_id in &canonical_e2e_scenario_ids {
            if !mapped_scenarios.contains(scenario_id) {
                violations.push(violation(
                    "OPS_SCENARIO_SLO_MAP_MISSING_SCENARIO",
                    format!("e2e scenario `{scenario_id}` missing from scenario-slo-map"),
                    "add mapping entries for every e2e scenario in ops/inventory/scenario-slo-map.json",
                    Some(scenario_slo_map_rel),
                ));
            }
        }
    } else {
        violations.push(violation(
            "OPS_SCENARIO_SLO_MAP_MISSING",
            format!(
                "missing scenario to slo mapping contract `{}`",
                scenario_slo_map_rel.display()
            ),
            "restore ops/inventory/scenario-slo-map.json",
            Some(scenario_slo_map_rel),
        ));
    }

    if ctx.adapters.fs.exists(ctx.repo_root, observe_coverage_rel)
        && ctx.adapters.fs.exists(ctx.repo_root, alerts_contract_rel)
    {
        let coverage_text = fs::read_to_string(ctx.repo_root.join(observe_coverage_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let coverage_json: serde_json::Value = serde_json::from_str(&coverage_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let coverage_entries = coverage_json
            .get("mappings")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let alerts_contract_text = fs::read_to_string(ctx.repo_root.join(alerts_contract_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let alerts_contract_json: serde_json::Value = serde_json::from_str(&alerts_contract_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let required_alerts = alerts_contract_json
            .get("required_alerts")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(ToString::to_string)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();

        let mut mapped_alerts = BTreeSet::new();
        for entry in &coverage_entries {
            let alert_id = entry
                .get("alert_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            if alert_id.is_empty() {
                continue;
            }
            mapped_alerts.insert(alert_id.clone());
            let slo_id = entry
                .get("slo_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if !slo_ids.contains(slo_id) {
                violations.push(violation(
                    "OPS_OBSERVABILITY_COVERAGE_UNKNOWN_SLO_ID",
                    format!(
                        "observability coverage entry `{alert_id}` references unknown slo id `{slo_id}`"
                    ),
                    "align observability-coverage-map slo_id values with ops/observe/slo-definitions.json",
                    Some(observe_coverage_rel),
                ));
            }
            let drill_id = entry
                .get("drill_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if !drill_ids.contains(drill_id) {
                violations.push(violation(
                    "OPS_OBSERVABILITY_COVERAGE_UNKNOWN_DRILL_ID",
                    format!(
                        "observability coverage entry `{alert_id}` references unknown drill id `{drill_id}`"
                    ),
                    "align observability-coverage-map drill_id values with ops/inventory/drills.json",
                    Some(observe_coverage_rel),
                ));
            }
        }
        for required_alert in &required_alerts {
            if !mapped_alerts.contains(required_alert) {
                violations.push(violation(
                    "OPS_OBSERVABILITY_COVERAGE_REQUIRED_ALERT_MISSING",
                    format!(
                        "required alert `{required_alert}` from alerts-contract is missing observability coverage mapping"
                    ),
                    "add alert coverage entry to ops/inventory/observability-coverage-map.json",
                    Some(observe_coverage_rel),
                ));
            }
        }
    } else if !ctx.adapters.fs.exists(ctx.repo_root, observe_coverage_rel) {
        violations.push(violation(
            "OPS_OBSERVABILITY_COVERAGE_MAP_MISSING",
            format!(
                "missing observability coverage map `{}`",
                observe_coverage_rel.display()
            ),
            "restore ops/inventory/observability-coverage-map.json",
            Some(observe_coverage_rel),
        ));
    }

    let realdata_readme_rel = Path::new("ops/e2e/realdata/README.md");
    if ctx.adapters.fs.exists(ctx.repo_root, realdata_readme_rel) {
        let text = fs::read_to_string(ctx.repo_root.join(realdata_readme_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if !(text.to_lowercase().contains("example") && text.to_lowercase().contains("required")) {
            violations.push(violation(
                "OPS_E2E_REALDATA_SNAPSHOT_POLICY_MISSING",
                "realdata README must distinguish example snapshots from required fixtures"
                    .to_string(),
                "document example vs required snapshot policy in ops/e2e/realdata/README.md",
                Some(realdata_readme_rel),
            ));
        }
    }

    let fixture_inventory_rel = Path::new("ops/datasets/generated/fixture-inventory.json");
    if !ctx.adapters.fs.exists(ctx.repo_root, fixture_inventory_rel) {
        violations.push(violation(
            "OPS_FIXTURE_INVENTORY_ARTIFACT_MISSING",
            format!(
                "missing fixture inventory generated artifact `{}`",
                fixture_inventory_rel.display()
            ),
            "generate and commit ops/datasets/generated/fixture-inventory.json",
            Some(fixture_inventory_rel),
        ));
    } else {
        let text = fs::read_to_string(ctx.repo_root.join(fixture_inventory_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let json: serde_json::Value =
            serde_json::from_str(&text).map_err(|err| CheckError::Failed(err.to_string()))?;
        let Some(fixtures) = json.get("fixtures").and_then(|v| v.as_array()) else {
            violations.push(violation(
                "OPS_FIXTURE_INVENTORY_SHAPE_INVALID",
                "fixture inventory must contain a fixtures array".to_string(),
                "populate fixtures array in fixture-inventory.json",
                Some(fixture_inventory_rel),
            ));
            return Ok(violations);
        };

        let mut indexed_versions = BTreeMap::new();
        for entry in fixtures {
            let Some(name) = entry.get("name").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(version) = entry.get("version").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(asset) = entry.get("asset").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(asset_sha) = entry.get("asset_sha256").and_then(|v| v.as_str()) else {
                violations.push(violation(
                    "OPS_FIXTURE_INVENTORY_HASH_MISSING",
                    format!("fixture inventory entry `{name}/{version}` is missing asset_sha256"),
                    "add asset_sha256 for each fixture inventory entry",
                    Some(fixture_inventory_rel),
                ));
                continue;
            };
            indexed_versions.insert(
                format!("{name}/{version}"),
                (asset.to_string(), asset_sha.to_string()),
            );
        }

        let mut discovered_versions = BTreeMap::new();
        for manifest in walk_files(&fixtures_root)
            .into_iter()
            .filter(|p| p.file_name().and_then(|v| v.to_str()) == Some("manifest.lock"))
        {
            let rel = manifest
                .strip_prefix(ctx.repo_root)
                .unwrap_or(manifest.as_path())
                .display()
                .to_string();
            let parts = rel.split('/').collect::<Vec<_>>();
            if parts.len() < 6 {
                continue;
            }
            let fixture_name = parts[3];
            let fixture_version = parts[4];
            let key = format!("{fixture_name}/{fixture_version}");
            let manifest_text =
                fs::read_to_string(&manifest).map_err(|err| CheckError::Failed(err.to_string()))?;
            let archive = manifest_text
                .lines()
                .find_map(|line| line.strip_prefix("archive="))
                .map(str::trim)
                .unwrap_or_default()
                .to_string();
            let manifest_sha = manifest_text
                .lines()
                .find_map(|line| line.strip_prefix("sha256="))
                .map(str::trim)
                .unwrap_or_default()
                .to_string();
            let asset =
                format!("ops/datasets/fixtures/{fixture_name}/{fixture_version}/assets/{archive}");
            let asset_path = ctx.repo_root.join(format!(
                "ops/datasets/fixtures/{fixture_name}/{fixture_version}/assets/{archive}"
            ));
            let asset_sha = if archive.is_empty() || !asset_path.exists() {
                String::new()
            } else {
                sha256_hex(&asset_path)?
            };
            if !manifest_sha.is_empty() && manifest_sha != asset_sha {
                violations.push(violation(
                    "OPS_FIXTURE_MANIFEST_SHA_STALE",
                    format!(
                        "manifest sha256 is stale for fixture `{key}`: manifest={} actual={}",
                        manifest_sha, asset_sha
                    ),
                    "refresh fixture manifest.lock sha256 after asset changes",
                    Some(Path::new(&rel)),
                ));
            }
            discovered_versions.insert(key, (asset, asset_sha));
        }

        for (key, (asset, sha)) in &discovered_versions {
            let Some((indexed_asset, indexed_sha)) = indexed_versions.get(key) else {
                violations.push(violation(
                    "OPS_FIXTURE_INVENTORY_ENTRY_MISSING",
                    format!("fixture inventory missing entry for `{key}`"),
                    "add fixture version entry to ops/datasets/generated/fixture-inventory.json",
                    Some(fixture_inventory_rel),
                ));
                continue;
            };
            if indexed_asset != asset {
                violations.push(violation(
                    "OPS_FIXTURE_INVENTORY_ASSET_PATH_DRIFT",
                    format!(
                        "fixture inventory asset path drift for `{key}`: expected `{asset}` got `{indexed_asset}`"
                    ),
                    "refresh fixture inventory asset paths from fixture manifests",
                    Some(fixture_inventory_rel),
                ));
            }
            if indexed_sha != sha {
                violations.push(violation(
                    "OPS_FIXTURE_INVENTORY_ASSET_HASH_DRIFT",
                    format!(
                        "fixture inventory hash drift for `{key}`: expected `{sha}` got `{indexed_sha}`"
                    ),
                    "refresh fixture inventory hashes from fixture assets",
                    Some(fixture_inventory_rel),
                ));
            }
        }
        for key in indexed_versions.keys() {
            if !discovered_versions.contains_key(key) {
                violations.push(violation(
                    "OPS_FIXTURE_INVENTORY_STALE_ENTRY",
                    format!("fixture inventory has stale entry `{key}`"),
                    "remove stale fixture inventory entries not backed by fixture manifests",
                    Some(fixture_inventory_rel),
                ));
            }
        }
    }

    let fixture_drift_rel = Path::new("ops/_generated.example/fixture-drift-report.json");
    if !ctx.adapters.fs.exists(ctx.repo_root, fixture_drift_rel) {
        violations.push(violation(
            "OPS_FIXTURE_DRIFT_REPORT_MISSING",
            format!(
                "missing fixture drift report artifact `{}`",
                fixture_drift_rel.display()
            ),
            "generate and commit fixture drift report under ops/_generated.example",
            Some(fixture_drift_rel),
        ));
    } else {
        let fixture_drift_text = fs::read_to_string(ctx.repo_root.join(fixture_drift_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let fixture_drift_json: serde_json::Value = serde_json::from_str(&fixture_drift_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        for key in ["schema_version", "generated_by", "status", "summary", "drift"] {
            if fixture_drift_json.get(key).is_none() {
                violations.push(violation(
                    "OPS_FIXTURE_DRIFT_REPORT_INVALID",
                    format!("fixture drift report is missing required key `{key}`"),
                    "populate fixture drift report with required governance keys",
                    Some(fixture_drift_rel),
                ));
            }
        }
        if !matches!(
            fixture_drift_json.get("status").and_then(|v| v.as_str()),
            Some("clean" | "pass")
        ) {
            violations.push(violation(
                "OPS_FIXTURE_DRIFT_REPORT_BLOCKING",
                "fixture drift report status must be `clean` or `pass`".to_string(),
                "resolve fixture drift and regenerate fixture-drift-report.json",
                Some(fixture_drift_rel),
            ));
        }
    }

    Ok(violations)
}

fn sha256_hex(path: &Path) -> Result<String, CheckError> {
    use sha2::{Digest, Sha256};
    let bytes = fs::read(path).map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    Ok(format!("{digest:x}"))
}

fn is_binary_like_file(path: &Path) -> Result<bool, CheckError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let known_binary_ext = [
        "gz", "zip", "zst", "tar", "sqlite", "db", "bin", "png", "jpg", "jpeg",
    ];
    if known_binary_ext.contains(&ext.as_str()) {
        return Ok(true);
    }
    let bytes = fs::read(path).map_err(|err| CheckError::Failed(err.to_string()))?;
    if bytes.contains(&0) {
        return Ok(true);
    }
    Ok(std::str::from_utf8(&bytes).is_err())
}

struct RequiredFilesContract {
    required_files: Vec<PathBuf>,
    required_dirs: Vec<PathBuf>,
    forbidden_patterns: Vec<String>,
    notes: Vec<String>,
}

fn parse_required_files_markdown_yaml(
    content: &str,
    rel: &Path,
) -> Result<RequiredFilesContract, CheckError> {
    let mut in_yaml = false;
    let mut yaml_block = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "```yaml" {
            in_yaml = true;
            continue;
        }
        if trimmed == "```" && in_yaml {
            break;
        }
        if in_yaml {
            yaml_block.push_str(line);
            yaml_block.push('\n');
        }
    }
    if yaml_block.trim().is_empty() {
        return Err(CheckError::Failed(format!(
            "{} must include a YAML contract block",
            rel.display()
        )));
    }
    let parsed: serde_yaml::Value =
        serde_yaml::from_str(&yaml_block).map_err(|err| CheckError::Failed(err.to_string()))?;
    let parsed_map = parsed.as_mapping().ok_or_else(|| {
        CheckError::Failed(format!(
            "{} YAML block must be a mapping with canonical keys",
            rel.display()
        ))
    })?;
    for key in [
        "required_files",
        "required_dirs",
        "forbidden_patterns",
        "notes",
    ] {
        if !parsed_map.contains_key(serde_yaml::Value::from(key)) {
            return Err(CheckError::Failed(format!(
                "{} must define `{key}` in REQUIRED_FILES contract YAML",
                rel.display()
            )));
        }
    }
    let required_files = parsed
        .get("required_files")
        .and_then(|v| v.as_sequence())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str())
                .map(PathBuf::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let required_dirs = parsed
        .get("required_dirs")
        .and_then(|v| v.as_sequence())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str())
                .map(PathBuf::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let forbidden_patterns = parsed
        .get("forbidden_patterns")
        .and_then(|v| v.as_sequence())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let notes = parsed
        .get("notes")
        .and_then(|v| v.as_sequence())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if required_files.is_empty() {
        return Err(CheckError::Failed(format!(
            "{} must define non-empty `required_files` YAML list",
            rel.display()
        )));
    }
    Ok(RequiredFilesContract {
        required_files,
        required_dirs,
        forbidden_patterns,
        notes,
    })
}

fn extract_ops_data_paths(text: &str) -> BTreeSet<String> {
    let mut refs = BTreeSet::new();
    for token in text.split_whitespace() {
        let trimmed = token
            .trim_matches(|c: char| {
                c == '`'
                    || c == '('
                    || c == ')'
                    || c == '['
                    || c == ']'
                    || c == ','
                    || c == ';'
                    || c == ':'
                    || c == '"'
                    || c == '\''
            })
            .to_string();
        if !trimmed.starts_with("ops/") {
            continue;
        }
        if trimmed.ends_with(".json")
            || trimmed.ends_with(".yaml")
            || trimmed.ends_with(".yml")
            || trimmed.ends_with(".toml")
        {
            refs.insert(trimmed);
        }
    }
    refs
}

pub(super) fn check_ops_quarantine_shim_expiration_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let contract_rel = Path::new("ops/CONTRACT.md");
    if !ctx.adapters.fs.exists(ctx.repo_root, contract_rel) {
        return Ok(vec![violation(
            "OPS_SHIM_QUARANTINE_README_MISSING",
            format!(
                "missing shim expiration contract file `{}`",
                contract_rel.display()
            ),
            "declare shim expiration deadline in ops contract",
            Some(contract_rel),
        )]);
    }
    let text = fs::read_to_string(ctx.repo_root.join(contract_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let deadline_line = text.lines().find(|line| {
        line.trim_start()
            .starts_with("- Legacy shell compatibility deadline: ")
    });
    let Some(deadline_line) = deadline_line else {
        return Ok(vec![violation(
            "OPS_SHIM_EXPIRATION_MISSING",
            "ops contract must declare an explicit shim expiration deadline".to_string(),
            "add a deadline line in the form `- Legacy shell compatibility deadline: YYYY-MM-DD.`",
            Some(contract_rel),
        )]);
    };
    let deadline = deadline_line
        .trim_start()
        .trim_start_matches("- Legacy shell compatibility deadline: ")
        .trim_end_matches('.')
        .trim();
    let valid_deadline = deadline.len() == 10
        && deadline.chars().enumerate().all(|(idx, ch)| match idx {
            4 | 7 => ch == '-',
            _ => ch.is_ascii_digit(),
        });
    if !valid_deadline {
        return Ok(vec![violation(
            "OPS_SHIM_EXPIRATION_FORMAT_INVALID",
            format!("shim quarantine deadline has invalid format: `{deadline}`"),
            "use ISO date format YYYY-MM-DD in shim quarantine deadline",
            Some(contract_rel),
        )]);
    }
    Ok(Vec::new())
}
