fn validate_load_and_report_manifests(
    repo_root: &Path,
    inputs: &LoadedOpsInventoryValidationInputs,
    errors: &mut Vec<String>,
) {
    let inventory = &inputs.inventory;
    let load_suites = &inputs.load_suites;
    let load_query_lock = &inputs.load_query_lock;
    let load_seed_policy = &inputs.load_seed_policy;
    let load_query_catalog = &inputs.load_query_catalog;
    let load_summary = &inputs.load_summary;
    let load_drift_report = &inputs.load_drift_report;
    if load_suites.schema_version != 2 {
        errors.push(format!(
            "{OPS_LOAD_SUITES_MANIFEST_PATH}: expected schema_version=2, got {}",
            load_suites.schema_version
        ));
    }
    if load_suites.suites.is_empty() {
        errors.push(format!(
            "{OPS_LOAD_SUITES_MANIFEST_PATH}: suites must not be empty"
        ));
    }
    if !repo_root.join(&load_suites.query_set).exists() {
        errors.push(format!(
            "{OPS_LOAD_SUITES_MANIFEST_PATH}: query_set path is missing `{}`",
            load_suites.query_set
        ));
    }
    let scenarios_dir = repo_root.join(&load_suites.scenarios_dir);
    if !scenarios_dir.exists() {
        errors.push(format!(
            "{OPS_LOAD_SUITES_MANIFEST_PATH}: scenarios_dir path is missing `{}`",
            load_suites.scenarios_dir
        ));
    }
    let mut suite_names = load_suites
        .suites
        .iter()
        .map(|suite| suite.name.clone())
        .collect::<Vec<_>>();
    let listed_suite_names_len = suite_names.len();
    suite_names.sort();
    suite_names.dedup();
    if listed_suite_names_len != suite_names.len() {
        errors.push(format!(
            "{OPS_LOAD_SUITES_MANIFEST_PATH}: suite names must be unique"
        ));
    }
    let legacy_manifest_dir = repo_root.join("ops/load/k6/manifests");
    if let Ok(entries) = fs::read_dir(&legacy_manifest_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext == "json")
            {
                errors.push(format!(
                    "ops/load/k6/manifests must not contain authored JSON (`{}`); move authored suites to {OPS_LOAD_SUITES_MANIFEST_PATH}",
                    path.strip_prefix(repo_root).unwrap_or(path.as_path()).display()
                ));
            }
        }
    }

    let canonical_thresholds_dir = repo_root.join("ops/load/thresholds");
    let mut canonical_threshold_filenames = BTreeSet::new();
    if let Ok(entries) = fs::read_dir(&canonical_thresholds_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext == "json")
            {
                if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                    canonical_threshold_filenames.insert(name.to_string());
                }
            }
        }
    }

    let legacy_thresholds_dir = repo_root.join("ops/load/k6/thresholds");
    if let Ok(entries) = fs::read_dir(&legacy_thresholds_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext == "json")
            {
                continue;
            }
            if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                if canonical_threshold_filenames.contains(name) {
                    errors.push(format!(
                        "duplicate thresholds filename `{name}` exists in both ops/load/thresholds and ops/load/k6/thresholds"
                    ));
                }
            }
        }
    }

    let mut expected_threshold_filenames = BTreeSet::new();
    let mut expected_scenarios = load_suites
        .suites
        .iter()
        .filter(|suite| suite.kind == "k6")
        .filter_map(|suite| suite.scenario.clone())
        .collect::<Vec<_>>();
    expected_scenarios.sort();
    expected_scenarios.dedup();
    let mut listed_covered = load_summary.scenario_coverage.covered.clone();
    let listed_missing = load_summary.scenario_coverage.missing.clone();
    for scenario in &expected_scenarios {
        if !repo_root
            .join(&load_suites.scenarios_dir)
            .join(scenario)
            .exists()
        {
            errors.push(format!(
                "{OPS_LOAD_SUITES_MANIFEST_PATH}: suite scenario is missing `{}`",
                scenario
            ));
        }
    }
    for suite in &load_suites.suites {
        let threshold_filename = format!("{}.thresholds.json", suite.name);
        expected_threshold_filenames.insert(threshold_filename.clone());
        let threshold_path = repo_root
            .join("ops/load/thresholds")
            .join(&threshold_filename);
        if !threshold_path.exists() {
            errors.push(format!(
                "{OPS_LOAD_SUITES_MANIFEST_PATH}: missing threshold file `{}` for suite `{}`",
                threshold_path
                    .strip_prefix(repo_root)
                    .unwrap_or(threshold_path.as_path())
                    .display(),
                suite.name
            ));
            continue;
        }
        if let Ok(text) = fs::read_to_string(&threshold_path) {
            if let Ok(threshold_json) = serde_json::from_str::<serde_json::Value>(&text) {
                let declared_suite = threshold_json
                    .get("suite")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default();
                if declared_suite != suite.name {
                    errors.push(format!(
                        "{}: suite field must be `{}`",
                        threshold_path
                            .strip_prefix(repo_root)
                            .unwrap_or(threshold_path.as_path())
                            .display(),
                        suite.name
                    ));
                }
            }
        }
        if suite.kind != "k6" {
            continue;
        }
        let Some(scenario_file) = suite.scenario.as_ref() else {
            errors.push(format!(
                "{OPS_LOAD_SUITES_MANIFEST_PATH}: k6 suite `{}` must define a scenario file",
                suite.name
            ));
            continue;
        };
        let scenario_path = repo_root
            .join(&load_suites.scenarios_dir)
            .join(scenario_file);
        if let Ok(text) = fs::read_to_string(&scenario_path) {
            if let Ok(scenario_json) = serde_json::from_str::<serde_json::Value>(&text) {
                let suite_script = scenario_json.get("suite").and_then(|value| value.as_str());
                match suite_script {
                    Some(script) if !script.trim().is_empty() => {
                        let script_path = repo_root.join("ops/load/k6/suites").join(script);
                        if !script_path.exists() {
                            errors.push(format!(
                                "{OPS_LOAD_SUITES_MANIFEST_PATH}: scenario `{}` for suite `{}` references missing script `ops/load/k6/suites/{}`",
                                scenario_file, suite.name, script
                            ));
                        }
                    }
                    _ => errors.push(format!(
                        "{OPS_LOAD_SUITES_MANIFEST_PATH}: scenario `{}` for suite `{}` must reference a k6 script via `suite`",
                        scenario_file, suite.name
                    )),
                }
            }
        }
    }
    for threshold_name in &canonical_threshold_filenames {
        if !expected_threshold_filenames.contains(threshold_name) {
            errors.push(format!(
                "unreferenced threshold file `ops/load/thresholds/{threshold_name}` is not mapped by any suite in {OPS_LOAD_SUITES_MANIFEST_PATH}"
            ));
        }
    }
    listed_covered.sort();
    listed_covered.dedup();
    if listed_covered != expected_scenarios {
        errors.push(format!(
            "{OPS_LOAD_SUMMARY_PATH}: scenario coverage mismatch, expected {expected_scenarios:?} got {listed_covered:?}"
        ));
    }
    if !listed_missing.is_empty() {
        errors.push(format!(
            "{OPS_LOAD_SUMMARY_PATH}: missing scenarios must be empty for stable load catalog"
        ));
    }
    if load_seed_policy.schema_version != 1 {
        errors.push(format!(
            "{OPS_LOAD_SEED_POLICY_PATH}: expected schema_version=1, got {}",
            load_seed_policy.schema_version
        ));
    }
    if load_query_lock.schema_version != 1 {
        errors.push(format!(
            "{OPS_LOAD_QUERY_LOCK_PATH}: expected schema_version=1, got {}",
            load_query_lock.schema_version
        ));
    }
    if load_query_lock.source != load_suites.query_set {
        errors.push(format!(
            "{OPS_LOAD_QUERY_LOCK_PATH}: source must match suite manifest query_set `{}`",
            load_suites.query_set
        ));
    }
    if load_query_lock.file_sha256.len() != 64
        || !load_query_lock
            .file_sha256
            .chars()
            .all(|ch| ch.is_ascii_hexdigit())
    {
        errors.push(format!(
            "{OPS_LOAD_QUERY_LOCK_PATH}: file_sha256 must be a 64-character hex digest"
        ));
    }
    if load_query_lock.query_hashes.is_empty() {
        errors.push(format!(
            "{OPS_LOAD_QUERY_LOCK_PATH}: query_hashes must not be empty"
        ));
    }
    if load_seed_policy.deterministic_seed == 0 {
        errors.push(format!(
            "{OPS_LOAD_SEED_POLICY_PATH}: deterministic_seed must be > 0"
        ));
    }
    if load_query_catalog.schema_version != 1 {
        errors.push(format!(
            "{OPS_LOAD_QUERY_PACK_CATALOG_PATH}: expected schema_version=1, got {}",
            load_query_catalog.schema_version
        ));
    }
    if load_query_catalog.packs.is_empty() {
        errors.push(format!(
            "{OPS_LOAD_QUERY_PACK_CATALOG_PATH}: packs must not be empty"
        ));
    }
    for pack in &load_query_catalog.packs {
        if pack.id.trim().is_empty() {
            errors.push(format!(
                "{OPS_LOAD_QUERY_PACK_CATALOG_PATH}: pack id must not be empty"
            ));
        }
        if !repo_root.join(&pack.query_file).exists() {
            errors.push(format!(
                "{OPS_LOAD_QUERY_PACK_CATALOG_PATH}: missing query_file `{}`",
                pack.query_file
            ));
        }
        if !repo_root.join(&pack.lock_file).exists() {
            errors.push(format!(
                "{OPS_LOAD_QUERY_PACK_CATALOG_PATH}: missing lock_file `{}`",
                pack.lock_file
            ));
        }
    }
    if load_summary.schema_version != 1 {
        errors.push(format!(
            "{OPS_LOAD_SUMMARY_PATH}: expected schema_version=1, got {}",
            load_summary.schema_version
        ));
    }
    if load_summary.query_pack.trim().is_empty() {
        errors.push(format!(
            "{OPS_LOAD_SUMMARY_PATH}: query_pack must not be empty"
        ));
    }
    if load_summary.deterministic_seed != load_seed_policy.deterministic_seed {
        errors.push(format!(
            "{OPS_LOAD_SUMMARY_PATH}: deterministic_seed must match {OPS_LOAD_SEED_POLICY_PATH}"
        ));
    }
    let mut summary_suites = load_summary.suites.clone();
    let listed_summary_suites = summary_suites.clone();
    summary_suites.sort();
    summary_suites.dedup();
    if listed_summary_suites != summary_suites {
        errors.push(format!(
            "{OPS_LOAD_SUMMARY_PATH}: suites must be unique and lexicographically sorted"
        ));
    }
    if summary_suites != suite_names {
        errors.push(format!(
            "{OPS_LOAD_SUMMARY_PATH}: suites mismatch from {OPS_LOAD_SUITES_MANIFEST_PATH}"
        ));
    }
    if load_drift_report.schema_version != 1 {
        errors.push(format!(
            "{OPS_LOAD_DRIFT_REPORT_PATH}: expected schema_version=1, got {}",
            load_drift_report.schema_version
        ));
    }
    if load_drift_report.status != "stable" {
        errors.push(format!(
            "{OPS_LOAD_DRIFT_REPORT_PATH}: status must be `stable`"
        ));
    }
    if load_drift_report.checks.is_empty() {
        errors.push(format!(
            "{OPS_LOAD_DRIFT_REPORT_PATH}: checks must not be empty"
        ));
    }
    let stack_generated_paths = [
        "ops/stack/generated/stack-index.json",
        "ops/stack/generated/dependency-graph.json",
        "ops/stack/generated/artifact-metadata.json",
    ];
    for rel in stack_generated_paths {
        if !repo_root.join(rel).exists() {
            errors.push(format!("missing required stack generated artifact `{rel}`"));
        }
    }

}
