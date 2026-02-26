pub fn validate_ops_inventory(repo_root: &Path) -> Vec<String> {
    let mut errors = Vec::new();

    for rel in [
        "ops/CONTRACT.md",
        "ops/ERRORS.md",
        "ops/INDEX.md",
        OPS_STACK_PROFILES_PATH,
        OPS_STACK_VERSION_MANIFEST_PATH,
        OPS_TOOLCHAIN_PATH,
        OPS_PINS_PATH,
        OPS_GATES_PATH,
        OPS_SURFACES_PATH,
        OPS_MIRROR_POLICY_PATH,
        OPS_CONTRACTS_PATH,
        OPS_K8S_INSTALL_MATRIX_PATH,
        OPS_K8S_CHART_PATH,
        OPS_OBSERVE_ALERT_CATALOG_PATH,
        OPS_OBSERVE_SLO_DEFINITIONS_PATH,
        OPS_OBSERVE_TELEMETRY_DRILLS_PATH,
        OPS_OBSERVE_READINESS_PATH,
        OPS_OBSERVE_TELEMETRY_INDEX_PATH,
        OPS_DATASETS_MANIFEST_PATH,
        OPS_DATASETS_MANIFEST_LOCK_PATH,
        OPS_DATASETS_PROMOTION_RULES_PATH,
        OPS_DATASETS_QC_METADATA_PATH,
        OPS_DATASETS_FIXTURE_POLICY_PATH,
        OPS_DATASETS_ROLLBACK_POLICY_PATH,
        OPS_DATASETS_INDEX_PATH,
        OPS_DATASETS_LINEAGE_PATH,
        OPS_E2E_SUITES_PATH,
        OPS_E2E_SCENARIOS_PATH,
        OPS_E2E_EXPECTATIONS_PATH,
        OPS_E2E_FIXTURE_ALLOWLIST_PATH,
        OPS_E2E_REPRODUCIBILITY_POLICY_PATH,
        OPS_E2E_TAXONOMY_PATH,
        OPS_E2E_SUMMARY_PATH,
        OPS_E2E_COVERAGE_MATRIX_PATH,
        OPS_REPORT_SCHEMA_PATH,
        OPS_REPORT_EVIDENCE_LEVELS_PATH,
        OPS_REPORT_EXAMPLE_PATH,
        OPS_REPORT_READINESS_SCORE_PATH,
        OPS_REPORT_DIFF_PATH,
        OPS_REPORT_HISTORY_PATH,
        OPS_REPORT_RELEASE_BUNDLE_PATH,
        OPS_LOAD_SUITES_MANIFEST_PATH,
        OPS_LOAD_QUERY_LOCK_PATH,
        OPS_LOAD_SEED_POLICY_PATH,
        OPS_LOAD_QUERY_PACK_CATALOG_PATH,
        OPS_LOAD_SUMMARY_PATH,
        OPS_LOAD_DRIFT_REPORT_PATH,
    ] {
        let path = repo_root.join(rel);
        if !path.exists() {
            errors.push(format!("missing required ops input `{rel}`"));
        }
    }

    let inventory = match load_ops_inventory(repo_root) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return errors;
        }
    };
    let k8s_install_matrix =
        match load_json::<K8sInstallMatrix>(repo_root, OPS_K8S_INSTALL_MATRIX_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return errors;
            }
        };
    let observe_alerts =
        match load_json::<ObserveCatalog>(repo_root, OPS_OBSERVE_ALERT_CATALOG_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return errors;
            }
        };
    let observe_slos =
        match load_json::<ObserveSloDefinitions>(repo_root, OPS_OBSERVE_SLO_DEFINITIONS_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return errors;
            }
        };
    let observe_drills =
        match load_json::<ObserveDrillCatalog>(repo_root, OPS_OBSERVE_TELEMETRY_DRILLS_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return errors;
            }
        };
    let observe_readiness =
        match load_json::<ObserveReadiness>(repo_root, OPS_OBSERVE_READINESS_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return errors;
            }
        };
    let observe_telemetry_index =
        match load_json::<ObserveTelemetryIndex>(repo_root, OPS_OBSERVE_TELEMETRY_INDEX_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return errors;
            }
        };
    let datasets_manifest_lock =
        match load_json::<DatasetManifestLock>(repo_root, OPS_DATASETS_MANIFEST_LOCK_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return errors;
            }
        };
    let datasets_promotion_rules =
        match load_json::<DatasetPromotionRules>(repo_root, OPS_DATASETS_PROMOTION_RULES_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return errors;
            }
        };
    let datasets_qc_metadata =
        match load_json::<DatasetQcMetadata>(repo_root, OPS_DATASETS_QC_METADATA_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return errors;
            }
        };
    let datasets_fixture_policy =
        match load_json::<DatasetFixturePolicy>(repo_root, OPS_DATASETS_FIXTURE_POLICY_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return errors;
            }
        };
    let datasets_rollback_policy =
        match load_json::<DatasetRollbackPolicy>(repo_root, OPS_DATASETS_ROLLBACK_POLICY_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return errors;
            }
        };
    let datasets_index = match load_json::<DatasetIndex>(repo_root, OPS_DATASETS_INDEX_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return errors;
        }
    };
    let datasets_lineage = match load_json::<DatasetLineage>(repo_root, OPS_DATASETS_LINEAGE_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return errors;
        }
    };
    let e2e_suites = match load_json::<E2eSuitesManifest>(repo_root, OPS_E2E_SUITES_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return errors;
        }
    };
    let e2e_scenarios = match load_json::<E2eScenariosManifest>(repo_root, OPS_E2E_SCENARIOS_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return errors;
        }
    };
    let e2e_expectations = match load_json::<E2eExpectations>(repo_root, OPS_E2E_EXPECTATIONS_PATH)
    {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return errors;
        }
    };
    let e2e_fixture_allowlist =
        match load_json::<E2eFixtureAllowlist>(repo_root, OPS_E2E_FIXTURE_ALLOWLIST_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return errors;
            }
        };
    let e2e_reproducibility =
        match load_json::<E2eReproducibilityPolicy>(repo_root, OPS_E2E_REPRODUCIBILITY_POLICY_PATH)
        {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return errors;
            }
        };
    let e2e_taxonomy = match load_json::<E2eTaxonomy>(repo_root, OPS_E2E_TAXONOMY_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return errors;
        }
    };
    let e2e_summary = match load_json::<E2eSummary>(repo_root, OPS_E2E_SUMMARY_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return errors;
        }
    };
    let e2e_coverage = match load_json::<E2eCoverageMatrix>(repo_root, OPS_E2E_COVERAGE_MATRIX_PATH)
    {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return errors;
        }
    };
    let report_evidence_levels =
        match load_json::<ReportEvidenceLevels>(repo_root, OPS_REPORT_EVIDENCE_LEVELS_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return errors;
            }
        };
    let report_readiness =
        match load_json::<ReportReadinessScore>(repo_root, OPS_REPORT_READINESS_SCORE_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return errors;
            }
        };
    let report_diff = match load_json::<ReportDiff>(repo_root, OPS_REPORT_DIFF_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return errors;
        }
    };
    let report_history =
        match load_json::<ReportHistoricalComparison>(repo_root, OPS_REPORT_HISTORY_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return errors;
            }
        };
    let report_bundle =
        match load_json::<ReportReleaseEvidenceBundle>(repo_root, OPS_REPORT_RELEASE_BUNDLE_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return errors;
            }
        };
    let load_suites =
        match load_json::<LoadSuitesManifest>(repo_root, OPS_LOAD_SUITES_MANIFEST_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return errors;
            }
        };
    let load_query_lock = match load_json::<LoadQueryLock>(repo_root, OPS_LOAD_QUERY_LOCK_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return errors;
        }
    };
    let load_seed_policy = match load_json::<LoadSeedPolicy>(repo_root, OPS_LOAD_SEED_POLICY_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return errors;
        }
    };
    let load_query_catalog =
        match load_json::<LoadQueryPackCatalog>(repo_root, OPS_LOAD_QUERY_PACK_CATALOG_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return errors;
            }
        };
    let load_summary = match load_json::<LoadSummary>(repo_root, OPS_LOAD_SUMMARY_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return errors;
        }
    };
    let load_drift_report =
        match load_json::<LoadDriftReport>(repo_root, OPS_LOAD_DRIFT_REPORT_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return errors;
            }
        };
    let pins_manifest = match load_pins_manifest(repo_root) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return errors;
        }
    };
    let gates_manifest = match load_json::<GatesManifest>(repo_root, OPS_GATES_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return errors;
        }
    };

    validate_pins_file_content(
        repo_root,
        inventory.toolchain.images.keys().cloned().collect(),
        inventory
            .stack_version_manifest
            .components
            .keys()
            .cloned()
            .collect(),
        &mut errors,
    );

    if inventory.stack_profiles.schema_version != EXPECTED_STACK_PROFILES_SCHEMA {
        errors.push(format!(
            "{OPS_STACK_PROFILES_PATH}: expected schema_version={EXPECTED_STACK_PROFILES_SCHEMA}, got {}",
            inventory.stack_profiles.schema_version
        ));
    }
    if inventory.stack_version_manifest.schema_version != EXPECTED_STACK_VERSION_SCHEMA {
        errors.push(format!(
            "{OPS_STACK_VERSION_MANIFEST_PATH}: expected schema_version={EXPECTED_STACK_VERSION_SCHEMA}, got {}",
            inventory.stack_version_manifest.schema_version
        ));
    }
    if inventory.toolchain.schema_version != EXPECTED_TOOLCHAIN_SCHEMA {
        errors.push(format!(
            "{OPS_TOOLCHAIN_PATH}: expected schema_version={EXPECTED_TOOLCHAIN_SCHEMA}, got {}",
            inventory.toolchain.schema_version
        ));
    }
    if inventory.surfaces.schema_version != EXPECTED_SURFACES_SCHEMA {
        errors.push(format!(
            "{OPS_SURFACES_PATH}: expected schema_version={EXPECTED_SURFACES_SCHEMA}, got {}",
            inventory.surfaces.schema_version
        ));
    }
    if inventory.mirror_policy.schema_version != EXPECTED_MIRROR_SCHEMA {
        errors.push(format!(
            "{OPS_MIRROR_POLICY_PATH}: expected schema_version={EXPECTED_MIRROR_SCHEMA}, got {}",
            inventory.mirror_policy.schema_version
        ));
    }
    if inventory.contracts.schema_version != EXPECTED_CONTRACTS_SCHEMA {
        errors.push(format!(
            "{OPS_CONTRACTS_PATH}: expected schema_version={EXPECTED_CONTRACTS_SCHEMA}, got {}",
            inventory.contracts.schema_version
        ));
    }
    if gates_manifest.schema_version != 1 {
        errors.push(format!(
            "{OPS_GATES_PATH}: expected schema_version=1, got {}",
            gates_manifest.schema_version
        ));
    }
    if gates_manifest.gates.is_empty() {
        errors.push(format!("{OPS_GATES_PATH}: gates must not be empty"));
    }
    let known_actions = inventory
        .surfaces
        .actions
        .iter()
        .map(|action| action.id.clone())
        .collect::<BTreeSet<_>>();
    let mut seen_gate_ids = BTreeSet::new();
    for gate in &gates_manifest.gates {
        if gate.id.trim().is_empty() {
            errors.push(format!("{OPS_GATES_PATH}: gate id must not be empty"));
            continue;
        }
        if !seen_gate_ids.insert(gate.id.clone()) {
            errors.push(format!("{OPS_GATES_PATH}: duplicate gate id `{}`", gate.id));
        }
        if gate.action_id.trim().is_empty() {
            errors.push(format!(
                "{OPS_GATES_PATH}: gate `{}` must define action_id",
                gate.id
            ));
            continue;
        }
        if !known_actions.contains(&gate.action_id) {
            errors.push(format!(
                "{OPS_GATES_PATH}: gate `{}` references unknown action_id `{}`",
                gate.id, gate.action_id
            ));
        }
    }
    for required in [
        "ops.doctor",
        "ops.validate",
        "ops.gate.directory-completeness",
        "ops.gate.schema-validation",
        "ops.gate.pin-drift",
        "ops.gate.stack-reproducibility",
        "ops.gate.k8s-determinism",
        "ops.gate.observe-coverage",
        "ops.gate.dataset-lifecycle",
        "ops.gate.unified-readiness",
    ] {
        if !seen_gate_ids.contains(required) {
            errors.push(format!(
                "{OPS_GATES_PATH}: missing required gate `{required}`"
            ));
        }
    }

    if inventory.stack_profiles.profiles.is_empty() {
        errors.push("ops stack profiles are empty".to_string());
    }

    let mut seen_profiles = BTreeSet::new();
    for profile in &inventory.stack_profiles.profiles {
        if !seen_profiles.insert(profile.name.clone()) {
            errors.push(format!(
                "{OPS_STACK_PROFILES_PATH}: duplicate profile `{}`",
                profile.name
            ));
        }
        if profile.kind_profile.trim().is_empty() {
            errors.push(format!(
                "{OPS_STACK_PROFILES_PATH}: profile `{}` has empty kind_profile",
                profile.name
            ));
        }
        let cluster_config = repo_root.join(&profile.cluster_config);
        if !cluster_config.exists() {
            errors.push(format!(
                "{OPS_STACK_PROFILES_PATH}: profile `{}` references missing cluster config `{}`",
                profile.name, profile.cluster_config
            ));
        }
    }
    for required_profile in ["minimal", "ci", "perf"] {
        if !inventory
            .stack_profiles
            .profiles
            .iter()
            .any(|profile| profile.name == required_profile)
        {
            errors.push(format!(
                "{OPS_STACK_PROFILES_PATH}: missing required profile `{required_profile}`"
            ));
        }
    }
    for required_cluster in [
        "ops/stack/kind/cluster-small.yaml",
        "ops/stack/kind/cluster-dev.yaml",
        "ops/stack/kind/cluster-perf.yaml",
    ] {
        if !repo_root.join(required_cluster).exists() {
            errors.push(format!(
                "{OPS_STACK_PROFILES_PATH}: missing required kind cluster config `{required_cluster}`"
            ));
        }
    }

    if inventory.toolchain.images.is_empty() {
        errors.push(format!(
            "{OPS_TOOLCHAIN_PATH}: images map must not be empty"
        ));
    }
    if inventory.toolchain.tools.is_empty() {
        errors.push(format!("{OPS_TOOLCHAIN_PATH}: tools map must not be empty"));
    }
    for (name, spec) in &inventory.toolchain.tools {
        if name.trim().is_empty() {
            errors.push(format!("{OPS_TOOLCHAIN_PATH}: tools key must not be empty"));
        }
        if spec.version_regex.trim().is_empty() {
            errors.push(format!(
                "{OPS_TOOLCHAIN_PATH}: tool `{name}` must define version_regex"
            ));
        }
        if spec.probe_argv.is_empty() {
            errors.push(format!(
                "{OPS_TOOLCHAIN_PATH}: tool `{name}` must define probe_argv"
            ));
        }
    }
    for (name, image) in &inventory.toolchain.images {
        if image.contains(":latest") {
            errors.push(format!(
                "{OPS_TOOLCHAIN_PATH}: image `{name}` uses forbidden latest tag `{image}`"
            ));
        }
    }
    if inventory.stack_version_manifest.components.is_empty() {
        errors.push(format!(
            "{OPS_STACK_VERSION_MANIFEST_PATH}: components must not be empty"
        ));
    }
    for (name, image) in &inventory.stack_version_manifest.components {
        if image.contains(":latest") {
            errors.push(format!(
                "{OPS_STACK_VERSION_MANIFEST_PATH}: component `{name}` uses forbidden latest tag `{image}`"
            ));
        }
    }
    for name in inventory.stack_version_manifest.components.keys() {
        if !inventory.toolchain.images.contains_key(name) {
            errors.push(format!(
                "pin coverage mismatch: `{name}` is present in {OPS_STACK_VERSION_MANIFEST_PATH} but missing in {OPS_TOOLCHAIN_PATH}"
            ));
        }
    }
    for name in inventory.toolchain.images.keys() {
        if !inventory
            .stack_version_manifest
            .components
            .contains_key(name)
        {
            errors.push(format!(
                "pin coverage mismatch: `{name}` is present in {OPS_TOOLCHAIN_PATH} but missing in {OPS_STACK_VERSION_MANIFEST_PATH}"
            ));
        }
    }
    for (name, image) in &pins_manifest.images {
        if inventory
            .toolchain
            .images
            .get(name)
            .is_some_and(|toolchain_image| toolchain_image != image)
        {
            errors.push(format!(
                "pin value drift: `{name}` differs between {OPS_PINS_PATH} and {OPS_TOOLCHAIN_PATH}"
            ));
        }
        if inventory
            .stack_version_manifest
            .components
            .get(name)
            .is_some_and(|stack_image| stack_image != image)
        {
            errors.push(format!(
                "pin value drift: `{name}` differs between {OPS_PINS_PATH} and {OPS_STACK_VERSION_MANIFEST_PATH}"
            ));
        }
    }
    if k8s_install_matrix.schema_version != 1 {
        errors.push(format!(
            "{OPS_K8S_INSTALL_MATRIX_PATH}: expected schema_version=1, got {}",
            k8s_install_matrix.schema_version
        ));
    }
    if k8s_install_matrix.profiles.is_empty() {
        errors.push(format!(
            "{OPS_K8S_INSTALL_MATRIX_PATH}: profiles must not be empty"
        ));
    }
    let names = k8s_install_matrix
        .profiles
        .iter()
        .map(|profile| profile.name.clone())
        .collect::<Vec<_>>();
    let mut sorted_names = names.clone();
    sorted_names.sort();
    sorted_names.dedup();
    if sorted_names != names {
        errors.push(format!(
            "{OPS_K8S_INSTALL_MATRIX_PATH}: profile names must be unique and lexicographically sorted"
        ));
    }
    for required in ["kind", "dev", "ci", "prod"] {
        if !k8s_install_matrix
            .profiles
            .iter()
            .any(|profile| profile.name == required)
        {
            errors.push(format!(
                "{OPS_K8S_INSTALL_MATRIX_PATH}: missing required install profile `{required}`"
            ));
        }
    }
    for profile in &k8s_install_matrix.profiles {
        if !repo_root.join(&profile.values_file).exists() {
            errors.push(format!(
                "{OPS_K8S_INSTALL_MATRIX_PATH}: profile `{}` references missing values file `{}`",
                profile.name, profile.values_file
            ));
        }
        if !matches!(
            profile.suite.as_str(),
            "install-gate" | "k8s-suite" | "nightly"
        ) {
            errors.push(format!(
                "{OPS_K8S_INSTALL_MATRIX_PATH}: profile `{}` uses unsupported suite `{}`",
                profile.name, profile.suite
            ));
        }
    }
    for rel in [
        "ops/k8s/generated/inventory-index.json",
        "ops/k8s/generated/render-artifact-index.json",
        "ops/k8s/generated/release-snapshot.json",
    ] {
        if !repo_root.join(rel).exists() {
            errors.push(format!("missing required k8s generated artifact `{rel}`"));
        }
    }
    if let Ok(chart_yaml) = fs::read_to_string(repo_root.join(OPS_K8S_CHART_PATH)) {
        if chart_yaml.contains("version: latest") || chart_yaml.contains("appVersion: \"latest\"") {
            errors.push(format!(
                "{OPS_K8S_CHART_PATH}: chart version and appVersion must be pinned and cannot be latest"
            ));
        }
    }
    for (name, version) in [
        ("alerts", observe_alerts.schema_version),
        ("slos", observe_slos.schema_version),
        ("drills", observe_drills.schema_version),
        ("readiness", observe_readiness.schema_version),
        ("telemetry-index", observe_telemetry_index.schema_version),
    ] {
        if version != 1 {
            errors.push(format!(
                "ops/observe: `{name}` manifest must use schema_version=1, got {version}"
            ));
        }
    }
    if observe_alerts.alerts.is_empty() {
        errors.push("ops/observe: alert catalog must not be empty".to_string());
    }
    if observe_slos.slos.is_empty() {
        errors.push("ops/observe: slo definitions must not be empty".to_string());
    }
    if observe_drills.drills.is_empty() {
        errors.push("ops/observe: telemetry drill catalog must not be empty".to_string());
    }
    if observe_readiness.status.trim() != "ready" {
        errors.push("ops/observe: readiness status must be `ready`".to_string());
    }
    for required in [
        "slo-definitions",
        "alert-catalog",
        "telemetry-drills",
        "dashboard-index",
    ] {
        if !observe_readiness
            .requirements
            .iter()
            .any(|entry| entry == required)
        {
            errors.push(format!(
                "ops/observe: readiness requirements missing `{required}`"
            ));
        }
    }
    let mut sorted_artifacts = observe_telemetry_index.artifacts.clone();
    let listed_artifacts = observe_telemetry_index.artifacts.clone();
    sorted_artifacts.sort();
    sorted_artifacts.dedup();
    if listed_artifacts != sorted_artifacts {
        errors.push(
            "ops/observe/generated/telemetry-index.json: artifacts must be unique and sorted"
                .to_string(),
        );
    }
    for artifact in &observe_telemetry_index.artifacts {
        if !repo_root.join(artifact).exists() {
            errors.push(format!(
                "ops/observe/generated/telemetry-index.json: missing referenced artifact `{artifact}`"
            ));
        }
    }
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

    let mut seen_action_ids = BTreeSet::new();
    for action in &inventory.surfaces.actions {
        if action.id.trim().is_empty() {
            errors.push(format!("{OPS_SURFACES_PATH}: action id must not be empty"));
            continue;
        }
        if !seen_action_ids.insert(action.id.clone()) {
            errors.push(format!(
                "{OPS_SURFACES_PATH}: duplicate action id `{}`",
                action.id
            ));
        }
        if action.domain.trim().is_empty() {
            errors.push(format!(
                "{OPS_SURFACES_PATH}: action `{}` has empty domain",
                action.id
            ));
        }
        if action.command.is_empty() {
            errors.push(format!(
                "{OPS_SURFACES_PATH}: action `{}` has empty command",
                action.id
            ));
        }
        let joined = action.command.join(" ");
        if joined.contains("scripts/") || joined.contains(".sh") {
            errors.push(format!(
                "{OPS_SURFACES_PATH}: action `{}` references forbidden script entrypoint `{joined}`",
                action.id
            ));
        }
    }

    for mirror in &inventory.mirror_policy.mirrors {
        if !repo_root.join(&mirror.committed).exists() {
            errors.push(format!(
                "{OPS_MIRROR_POLICY_PATH}: committed path missing `{}`",
                mirror.committed
            ));
        }
        if !mirror.source.starts_with("ops/_generated/") && !repo_root.join(&mirror.source).exists()
        {
            errors.push(format!(
                "{OPS_MIRROR_POLICY_PATH}: source path missing `{}`",
                mirror.source
            ));
        }
    }
    let sorted_mirror_keys = inventory
        .mirror_policy
        .mirrors
        .iter()
        .map(|entry| entry.committed.clone())
        .collect::<Vec<_>>();
    let mut dedup = sorted_mirror_keys.clone();
    dedup.sort();
    dedup.dedup();
    if dedup.len() != sorted_mirror_keys.len() {
        errors.push(format!(
            "{OPS_MIRROR_POLICY_PATH}: mirror committed paths must be unique"
        ));
    }
    if sorted_mirror_keys != dedup {
        errors.push(format!(
            "{OPS_MIRROR_POLICY_PATH}: mirror committed paths must be sorted for deterministic output"
        ));
    }

    let allowed_top_level: BTreeSet<&str> = [
        "_generated",
        "_generated.example",
        "_meta",
        "atlas-dev",
        "datasets",
        "docs",
        "e2e",
        "env",
        "fixtures",
        "inventory",
        "k8s",
        "load",
        "observe",
        "report",
        "schema",
        "stack",
        "tools",
        "vendor",
        "CONTRACT.md",
        "CONTROL_PLANE.md",
        "DIRECTORY_BUDGET_POLICY.md",
        "DRIFT.md",
        "ERRORS.md",
        "GENERATED_LIFECYCLE.md",
        "INDEX.md",
        "NAMING.md",
        "README.md",
        "SSOT.md",
    ]
    .into_iter()
    .collect();
    if let Ok(entries) = fs::read_dir(repo_root.join("ops")) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !allowed_top_level.contains(name.as_ref()) {
                errors.push(format!("unexpected ops top-level entry `ops/{name}`"));
            }
        }
    }

    let bash_like = fs::read_dir(repo_root.join("ops"))
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .flat_map(|entry| collect_files_recursive(entry.path()))
        .filter(|path| {
            path.extension()
                .and_then(|v| v.to_str())
                .is_some_and(|ext| ext == "sh" || ext == "bash")
        })
        .collect::<Vec<_>>();
    for path in bash_like {
        let rel = path
            .strip_prefix(repo_root)
            .unwrap_or(path.as_path())
            .display()
            .to_string();
        errors.push(format!(
            "forbidden bash helper outside rust control-plane: `{rel}`"
        ));
    }

    if repo_root.join("ops/_lib").exists() {
        errors.push("forbidden retired path exists: ops/_lib".to_string());
    }
    if repo_root.join("ops/schema/obs").exists() {
        errors.push("forbidden retired path exists: ops/schema/obs".to_string());
    }
    if let Ok(entries) = fs::read_dir(repo_root.join("ops/inventory/contracts")) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.contains("obs.") {
                errors.push(format!(
                    "forbidden retired contract fragment name in ops/inventory/contracts: `{file_name}`"
                ));
            }
        }
    }

    errors.sort();
    errors.dedup();
    errors
}

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

