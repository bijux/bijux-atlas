fn validate_stack_k8s_and_observe_manifests(
    repo_root: &Path,
    inputs: &LoadedOpsInventoryValidationInputs,
    errors: &mut Vec<String>,
) {
    let inventory = &inputs.inventory;
    let k8s_install_matrix = &inputs.k8s_install_matrix;
    let observe_alerts = &inputs.observe_alerts;
    let observe_slos = &inputs.observe_slos;
    let observe_drills = &inputs.observe_drills;
    let observe_readiness = &inputs.observe_readiness;
    let observe_telemetry_index = &inputs.observe_telemetry_index;
    let _datasets_manifest_lock = &inputs.datasets_manifest_lock;
    let _datasets_promotion_rules = &inputs.datasets_promotion_rules;
    let _datasets_qc_metadata = &inputs.datasets_qc_metadata;
    let _datasets_fixture_policy = &inputs.datasets_fixture_policy;
    let _datasets_rollback_policy = &inputs.datasets_rollback_policy;
    let _datasets_index = &inputs.datasets_index;
    let _datasets_lineage = &inputs.datasets_lineage;
    let _e2e_suites = &inputs.e2e_suites;
    let _e2e_scenarios = &inputs.e2e_scenarios;
    let _e2e_expectations = &inputs.e2e_expectations;
    let _e2e_fixture_allowlist = &inputs.e2e_fixture_allowlist;
    let _e2e_reproducibility = &inputs.e2e_reproducibility;
    let _e2e_taxonomy = &inputs.e2e_taxonomy;
    let _e2e_summary = &inputs.e2e_summary;
    let _e2e_coverage = &inputs.e2e_coverage;
    let _report_evidence_levels = &inputs.report_evidence_levels;
    let _report_readiness = &inputs.report_readiness;
    let _report_diff = &inputs.report_diff;
    let _report_history = &inputs.report_history;
    let _report_bundle = &inputs.report_bundle;
    let _load_suites = &inputs.load_suites;
    let _load_query_lock = &inputs.load_query_lock;
    let _load_seed_policy = &inputs.load_seed_policy;
    let _load_query_catalog = &inputs.load_query_catalog;
    let _load_summary = &inputs.load_summary;
    let _load_drift_report = &inputs.load_drift_report;
    let pins_manifest = &inputs.pins_manifest;
    let gates_manifest = &inputs.gates_manifest;
    validate_pins_file_content(
        repo_root,
        inventory.toolchain.images.keys().cloned().collect(),
        inventory
            .stack_version_manifest
            .components
            .keys()
            .cloned()
            .collect(),
        errors,
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
}
