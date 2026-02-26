fn validate_datasets_and_e2e_manifests(
    repo_root: &Path,
    inputs: &LoadedOpsInventoryValidationInputs,
    errors: &mut Vec<String>,
) {
    let _inventory = &inputs.inventory;
    let datasets_manifest_lock = &inputs.datasets_manifest_lock;
    let datasets_promotion_rules = &inputs.datasets_promotion_rules;
    let datasets_qc_metadata = &inputs.datasets_qc_metadata;
    let datasets_fixture_policy = &inputs.datasets_fixture_policy;
    let datasets_rollback_policy = &inputs.datasets_rollback_policy;
    let datasets_index = &inputs.datasets_index;
    let datasets_lineage = &inputs.datasets_lineage;
    let e2e_suites = &inputs.e2e_suites;
    let e2e_scenarios = &inputs.e2e_scenarios;
    let e2e_expectations = &inputs.e2e_expectations;
    let e2e_fixture_allowlist = &inputs.e2e_fixture_allowlist;
    let e2e_reproducibility = &inputs.e2e_reproducibility;
    let e2e_taxonomy = &inputs.e2e_taxonomy;
    let e2e_summary = &inputs.e2e_summary;
    let e2e_coverage = &inputs.e2e_coverage;
    let report_evidence_levels = &inputs.report_evidence_levels;
    let report_readiness = &inputs.report_readiness;
    let report_diff = &inputs.report_diff;
    let report_history = &inputs.report_history;
    let report_bundle = &inputs.report_bundle;
    let _load_suites = &inputs.load_suites;
    let _load_query_lock = &inputs.load_query_lock;
    let _load_seed_policy = &inputs.load_seed_policy;
    let _load_query_catalog = &inputs.load_query_catalog;
    let _load_summary = &inputs.load_summary;
    let _load_drift_report = &inputs.load_drift_report;
    let manifest_ids = match load_json::<DatasetsManifest>(repo_root, OPS_DATASETS_MANIFEST_PATH) {
        Ok(manifest) => {
            if manifest.schema_version < 1 {
                errors.push(format!(
                    "{OPS_DATASETS_MANIFEST_PATH}: schema_version must be >= 1"
                ));
            }
            manifest
                .datasets
                .iter()
                .map(|entry| entry.id.clone())
                .collect::<BTreeSet<_>>()
        }
        Err(err) => {
            errors.push(err);
            BTreeSet::new()
        }
    };
    if datasets_manifest_lock.schema_version != 1 {
        errors.push(format!(
            "{OPS_DATASETS_MANIFEST_LOCK_PATH}: expected schema_version=1, got {}",
            datasets_manifest_lock.schema_version
        ));
    }
    let locked_ids = datasets_manifest_lock
        .entries
        .iter()
        .map(|entry| entry.id.clone())
        .collect::<BTreeSet<_>>();
    if locked_ids != manifest_ids {
        errors.push(format!(
            "{OPS_DATASETS_MANIFEST_LOCK_PATH}: manifest lock ids must match {OPS_DATASETS_MANIFEST_PATH}"
        ));
    }
    if datasets_promotion_rules.schema_version != 1 {
        errors.push(format!(
            "{OPS_DATASETS_PROMOTION_RULES_PATH}: expected schema_version=1, got {}",
            datasets_promotion_rules.schema_version
        ));
    }
    if datasets_promotion_rules.pins_source != OPS_PINS_PATH {
        errors.push(format!(
            "{OPS_DATASETS_PROMOTION_RULES_PATH}: pins_source must be `{OPS_PINS_PATH}`"
        ));
    }
    if datasets_promotion_rules.manifest_lock != OPS_DATASETS_MANIFEST_LOCK_PATH {
        errors.push(format!(
            "{OPS_DATASETS_PROMOTION_RULES_PATH}: manifest_lock must be `{OPS_DATASETS_MANIFEST_LOCK_PATH}`"
        ));
    }
    if datasets_promotion_rules.environments.is_empty() {
        errors.push(format!(
            "{OPS_DATASETS_PROMOTION_RULES_PATH}: environments must not be empty"
        ));
    }
    if datasets_qc_metadata.schema_version != 1 {
        errors.push(format!(
            "{OPS_DATASETS_QC_METADATA_PATH}: expected schema_version=1, got {}",
            datasets_qc_metadata.schema_version
        ));
    }
    if datasets_qc_metadata.stale_after_days == 0 {
        errors.push(format!(
            "{OPS_DATASETS_QC_METADATA_PATH}: stale_after_days must be > 0"
        ));
    }
    if !repo_root
        .join(&datasets_qc_metadata.golden_summary)
        .exists()
    {
        errors.push(format!(
            "{OPS_DATASETS_QC_METADATA_PATH}: golden_summary path is missing `{}`",
            datasets_qc_metadata.golden_summary
        ));
    }
    if datasets_fixture_policy.schema_version != 1 {
        errors.push(format!(
            "{OPS_DATASETS_FIXTURE_POLICY_PATH}: expected schema_version=1, got {}",
            datasets_fixture_policy.schema_version
        ));
    }
    if datasets_fixture_policy.fixture_roots.is_empty() {
        errors.push(format!(
            "{OPS_DATASETS_FIXTURE_POLICY_PATH}: fixture_roots must not be empty"
        ));
    }
    if datasets_fixture_policy.allow_remote_download {
        errors.push(format!(
            "{OPS_DATASETS_FIXTURE_POLICY_PATH}: allow_remote_download must be false"
        ));
    }
    for root in &datasets_fixture_policy.fixture_roots {
        if !repo_root.join(root).exists() {
            errors.push(format!(
                "{OPS_DATASETS_FIXTURE_POLICY_PATH}: fixture root is missing `{root}`"
            ));
        }
    }
    if datasets_rollback_policy.schema_version != 1 {
        errors.push(format!(
            "{OPS_DATASETS_ROLLBACK_POLICY_PATH}: expected schema_version=1, got {}",
            datasets_rollback_policy.schema_version
        ));
    }
    if datasets_rollback_policy.strategy.trim().is_empty() {
        errors.push(format!(
            "{OPS_DATASETS_ROLLBACK_POLICY_PATH}: strategy must not be empty"
        ));
    }
    if datasets_rollback_policy.rollback_steps.is_empty() {
        errors.push(format!(
            "{OPS_DATASETS_ROLLBACK_POLICY_PATH}: rollback_steps must not be empty"
        ));
    }
    if datasets_index.schema_version != 1 {
        errors.push(format!(
            "{OPS_DATASETS_INDEX_PATH}: expected schema_version=1, got {}",
            datasets_index.schema_version
        ));
    }
    let indexed_ids = datasets_index
        .dataset_ids
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if indexed_ids != manifest_ids {
        errors.push(format!(
            "{OPS_DATASETS_INDEX_PATH}: dataset_ids must match {OPS_DATASETS_MANIFEST_PATH}"
        ));
    }
    if !datasets_index.missing_dataset_ids.is_empty() {
        errors.push(format!(
            "{OPS_DATASETS_INDEX_PATH}: missing_dataset_ids must be empty"
        ));
    }
    if !datasets_index.stale_dataset_ids.is_empty() {
        errors.push(format!(
            "{OPS_DATASETS_INDEX_PATH}: stale_dataset_ids must be empty"
        ));
    }
    if datasets_lineage.schema_version != 1 {
        errors.push(format!(
            "{OPS_DATASETS_LINEAGE_PATH}: expected schema_version=1, got {}",
            datasets_lineage.schema_version
        ));
    }
    if datasets_lineage.nodes.is_empty() {
        errors.push(format!(
            "{OPS_DATASETS_LINEAGE_PATH}: nodes must not be empty"
        ));
    }
    let node_ids = datasets_lineage
        .nodes
        .iter()
        .map(|node| node.id.clone())
        .collect::<BTreeSet<_>>();
    if node_ids != manifest_ids {
        errors.push(format!(
            "{OPS_DATASETS_LINEAGE_PATH}: lineage nodes must match {OPS_DATASETS_MANIFEST_PATH}"
        ));
    }
    for edge in &datasets_lineage.edges {
        if !node_ids.contains(&edge.from) || !node_ids.contains(&edge.to) {
            errors.push(format!(
                "{OPS_DATASETS_LINEAGE_PATH}: edge `{} -> {}` references unknown dataset node",
                edge.from, edge.to
            ));
        }
    }
    if e2e_suites.schema_version != 1 {
        errors.push(format!(
            "{OPS_E2E_SUITES_PATH}: expected schema_version=1, got {}",
            e2e_suites.schema_version
        ));
    }
    if e2e_scenarios.schema_version != 1 {
        errors.push(format!(
            "{OPS_E2E_SCENARIOS_PATH}: expected schema_version=1, got {}",
            e2e_scenarios.schema_version
        ));
    }
    if e2e_expectations.schema_version != 1 {
        errors.push(format!(
            "{OPS_E2E_EXPECTATIONS_PATH}: expected schema_version=1, got {}",
            e2e_expectations.schema_version
        ));
    }
    if e2e_fixture_allowlist.schema_version != 1 {
        errors.push(format!(
            "{OPS_E2E_FIXTURE_ALLOWLIST_PATH}: expected schema_version=1, got {}",
            e2e_fixture_allowlist.schema_version
        ));
    }
    if e2e_reproducibility.schema_version != 1 {
        errors.push(format!(
            "{OPS_E2E_REPRODUCIBILITY_POLICY_PATH}: expected schema_version=1, got {}",
            e2e_reproducibility.schema_version
        ));
    }
    if e2e_taxonomy.schema_version != 1 {
        errors.push(format!(
            "{OPS_E2E_TAXONOMY_PATH}: expected schema_version=1, got {}",
            e2e_taxonomy.schema_version
        ));
    }
    if e2e_summary.schema_version != 1 {
        errors.push(format!(
            "{OPS_E2E_SUMMARY_PATH}: expected schema_version=1, got {}",
            e2e_summary.schema_version
        ));
    }
    if e2e_coverage.schema_version != 1 {
        errors.push(format!(
            "{OPS_E2E_COVERAGE_MATRIX_PATH}: expected schema_version=1, got {}",
            e2e_coverage.schema_version
        ));
    }
    if e2e_suites.suites.is_empty() {
        errors.push(format!("{OPS_E2E_SUITES_PATH}: suites must not be empty"));
    }
    if e2e_scenarios.scenarios.is_empty() {
        errors.push(format!(
            "{OPS_E2E_SCENARIOS_PATH}: scenarios must not be empty"
        ));
    }
    let suite_ids = e2e_suites
        .suites
        .iter()
        .map(|suite| suite.id.clone())
        .collect::<BTreeSet<_>>();
    let scenario_ids = e2e_scenarios
        .scenarios
        .iter()
        .map(|scenario| scenario.id.clone())
        .collect::<BTreeSet<_>>();
    let expectation_ids = e2e_expectations
        .expectations
        .iter()
        .map(|entry| entry.scenario_id.clone())
        .collect::<BTreeSet<_>>();
    if expectation_ids != scenario_ids {
        errors.push(format!(
            "{OPS_E2E_EXPECTATIONS_PATH}: expectation scenario_ids must exactly match {OPS_E2E_SCENARIOS_PATH}"
        ));
    }
    let allowed_compose_keys: BTreeSet<&str> = ["stack", "observe", "datasets", "load", "k8s"]
        .into_iter()
        .collect();
    for scenario in &e2e_scenarios.scenarios {
        if scenario
            .action_id
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
        {
            errors.push(format!(
                "{OPS_E2E_SCENARIOS_PATH}: scenario `{}` must define action_id",
                scenario.id
            ));
        }
        for key in scenario.compose.keys() {
            if !allowed_compose_keys.contains(key.as_str()) {
                errors.push(format!(
                    "{OPS_E2E_SCENARIOS_PATH}: scenario `{}` compose key `{}` is not allowed",
                    scenario.id, key
                ));
            }
        }
    }
    for suite in &e2e_suites.suites {
        if suite.required_capabilities.is_empty() {
            errors.push(format!(
                "{OPS_E2E_SUITES_PATH}: suite `{}` must define required_capabilities",
                suite.id
            ));
        }
    }
    if e2e_fixture_allowlist.allowed_paths.is_empty() {
        errors.push(format!(
            "{OPS_E2E_FIXTURE_ALLOWLIST_PATH}: allowed_paths must not be empty"
        ));
    }
    let allowlisted = e2e_fixture_allowlist
        .allowed_paths
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for file in collect_files_recursive(repo_root.join("ops/e2e/fixtures")) {
        let rel = file
            .strip_prefix(repo_root)
            .unwrap_or(file.as_path())
            .display()
            .to_string();
        if !allowlisted.contains(&rel) {
            errors.push(format!(
                "{OPS_E2E_FIXTURE_ALLOWLIST_PATH}: file `{rel}` is not allowlisted"
            ));
        }
    }
    if e2e_reproducibility.ordering != "stable" {
        errors.push(format!(
            "{OPS_E2E_REPRODUCIBILITY_POLICY_PATH}: ordering must be `stable`"
        ));
    }
    if !repo_root.join(&e2e_reproducibility.seed_source).exists() {
        errors.push(format!(
            "{OPS_E2E_REPRODUCIBILITY_POLICY_PATH}: seed_source path is missing `{}`",
            e2e_reproducibility.seed_source
        ));
    }
    if e2e_taxonomy.categories.is_empty() {
        errors.push(format!(
            "{OPS_E2E_TAXONOMY_PATH}: categories must not be empty"
        ));
    }
    let taxonomy_ids = e2e_taxonomy
        .categories
        .iter()
        .map(|entry| entry.id.clone())
        .collect::<BTreeSet<_>>();
    for expected in ["smoke", "kubernetes", "realdata", "performance"] {
        if !taxonomy_ids.contains(expected) {
            errors.push(format!(
                "{OPS_E2E_TAXONOMY_PATH}: missing expected category `{expected}`"
            ));
        }
    }
    if e2e_summary.status != "stable" {
        errors.push(format!("{OPS_E2E_SUMMARY_PATH}: status must be `stable`"));
    }
    if e2e_summary.suite_count != suite_ids.len() as u64 {
        errors.push(format!(
            "{OPS_E2E_SUMMARY_PATH}: suite_count must match suite ids count"
        ));
    }
    if e2e_summary.scenario_count != scenario_ids.len() as u64 {
        errors.push(format!(
            "{OPS_E2E_SUMMARY_PATH}: scenario_count must match scenario ids count"
        ));
    }
    if e2e_summary
        .suite_ids
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        != suite_ids
    {
        errors.push(format!(
            "{OPS_E2E_SUMMARY_PATH}: suite_ids must match {OPS_E2E_SUITES_PATH}"
        ));
    }
    if e2e_summary
        .scenario_ids
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        != scenario_ids
    {
        errors.push(format!(
            "{OPS_E2E_SUMMARY_PATH}: scenario_ids must match {OPS_E2E_SCENARIOS_PATH}"
        ));
    }
    if !e2e_coverage.missing_domains.is_empty() {
        errors.push(format!(
            "{OPS_E2E_COVERAGE_MATRIX_PATH}: missing_domains must be empty"
        ));
    }
    let covered_scenarios = e2e_coverage
        .rows
        .iter()
        .map(|row| row.scenario_id.clone())
        .collect::<BTreeSet<_>>();
    if covered_scenarios != scenario_ids {
        errors.push(format!(
            "{OPS_E2E_COVERAGE_MATRIX_PATH}: coverage rows must match scenario ids"
        ));
    }
    for row in &e2e_coverage.rows {
        if row.covers.is_empty() {
            errors.push(format!(
                "{OPS_E2E_COVERAGE_MATRIX_PATH}: scenario `{}` must cover at least one domain",
                row.scenario_id
            ));
        }
    }
    if report_evidence_levels.schema_version != 1 {
        errors.push(format!(
            "{OPS_REPORT_EVIDENCE_LEVELS_PATH}: expected schema_version=1, got {}",
            report_evidence_levels.schema_version
        ));
    }
    let report_levels = report_evidence_levels
        .levels
        .iter()
        .map(|entry| entry.id.clone())
        .collect::<BTreeSet<_>>();
    for required in ["minimal", "standard", "forensic"] {
        if !report_levels.contains(required) {
            errors.push(format!(
                "{OPS_REPORT_EVIDENCE_LEVELS_PATH}: missing required level `{required}`"
            ));
        }
    }
    if report_readiness.schema_version != 1 {
        errors.push(format!(
            "{OPS_REPORT_READINESS_SCORE_PATH}: expected schema_version=1, got {}",
            report_readiness.schema_version
        ));
    }
    if report_readiness.score > 100 {
        errors.push(format!(
            "{OPS_REPORT_READINESS_SCORE_PATH}: score must be between 0 and 100"
        ));
    }
    if report_readiness.status != "ready" && report_readiness.status != "blocked" {
        errors.push(format!(
            "{OPS_REPORT_READINESS_SCORE_PATH}: status must be `ready` or `blocked`"
        ));
    }
    if report_diff.schema_version != 1 {
        errors.push(format!(
            "{OPS_REPORT_DIFF_PATH}: expected schema_version=1, got {}",
            report_diff.schema_version
        ));
    }
    if report_diff.status != "stable" && report_diff.status != "changed" {
        errors.push(format!(
            "{OPS_REPORT_DIFF_PATH}: status must be `stable` or `changed`"
        ));
    }
    if report_history.schema_version != 1 {
        errors.push(format!(
            "{OPS_REPORT_HISTORY_PATH}: expected schema_version=1, got {}",
            report_history.schema_version
        ));
    }
    if report_history.status != "stable" && report_history.status != "regressed" {
        errors.push(format!(
            "{OPS_REPORT_HISTORY_PATH}: status must be `stable` or `regressed`"
        ));
    }
    if !matches!(report_history.trend.as_str(), "up" | "flat" | "down") {
        errors.push(format!(
            "{OPS_REPORT_HISTORY_PATH}: trend must be one of `up`, `flat`, `down`"
        ));
    }
    if report_bundle.schema_version != 1 {
        errors.push(format!(
            "{OPS_REPORT_RELEASE_BUNDLE_PATH}: expected schema_version=1, got {}",
            report_bundle.schema_version
        ));
    }
    if report_bundle.status != "ready" && report_bundle.status != "blocked" {
        errors.push(format!(
            "{OPS_REPORT_RELEASE_BUNDLE_PATH}: status must be `ready` or `blocked`"
        ));
    }
    if report_bundle.bundle_paths.is_empty() {
        errors.push(format!(
            "{OPS_REPORT_RELEASE_BUNDLE_PATH}: bundle_paths must not be empty"
        ));
    }
    for path in &report_bundle.bundle_paths {
        if !repo_root.join(path).exists() {
            errors.push(format!(
                "{OPS_REPORT_RELEASE_BUNDLE_PATH}: bundle path is missing `{path}`"
            ));
        }
    }
    if !repo_root.join(OPS_REPORT_SCHEMA_PATH).exists() {
        errors.push(format!(
            "missing required report schema `{OPS_REPORT_SCHEMA_PATH}`"
        ));
    }
    if !repo_root.join(OPS_REPORT_EXAMPLE_PATH).exists() {
        errors.push(format!(
            "missing required report example `{OPS_REPORT_EXAMPLE_PATH}`"
        ));
    }
}
