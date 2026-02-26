fn validate_e2e_fixture_and_mapping_governance(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
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

    let incident_template_rel = Path::new("ops/observe/drills/templates/incident-template.md");
    if ctx.adapters.fs.exists(ctx.repo_root, incident_template_rel) {
        let text = fs::read_to_string(ctx.repo_root.join(incident_template_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        for required in ["- Dashboard Links:", "- Runbook Links:", "- Drill Result Path:"] {
            if !text.contains(required) {
                violations.push(violation(
                    "OPS_INCIDENT_TEMPLATE_LINKAGE_FIELD_MISSING",
                    format!(
                        "incident drill template must include linkage field `{required}`"
                    ),
                    "add dashboard, runbook, and drill result path linkage fields to incident template",
                    Some(incident_template_rel),
                ));
            }
        }
    }

    let golden_refresh_policy_rel = Path::new("ops/GOLDEN_REFRESH_POLICY.md");
    if !ctx.adapters.fs.exists(ctx.repo_root, golden_refresh_policy_rel) {
        violations.push(violation(
            "OPS_GOLDEN_REFRESH_POLICY_MISSING",
            "missing golden refresh policy `ops/GOLDEN_REFRESH_POLICY.md`".to_string(),
            "add a golden refresh policy with approvers, regeneration commands, and review expectations",
            Some(golden_refresh_policy_rel),
        ));
    } else {
        let text = fs::read_to_string(ctx.repo_root.join(golden_refresh_policy_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        for required in ["## Scope", "## Regeneration", "## Review", "## Approval"] {
            if !text.contains(required) {
                violations.push(violation(
                    "OPS_GOLDEN_REFRESH_POLICY_INCOMPLETE",
                    format!("golden refresh policy is missing `{required}`"),
                    "document scope, regeneration, review, and approval sections",
                    Some(golden_refresh_policy_rel),
                ));
            }
        }
    }

    let e2e_invariants_rel = Path::new("ops/e2e/END_TO_END_INVARIANTS.md");
    if !ctx.adapters.fs.exists(ctx.repo_root, e2e_invariants_rel) {
        violations.push(violation(
            "OPS_E2E_INVARIANTS_CONTRACT_MISSING",
            "missing end-to-end invariants contract `ops/e2e/END_TO_END_INVARIANTS.md`".to_string(),
            "add a deterministic end-to-end invariants contract with at least five must-pass invariants",
            Some(e2e_invariants_rel),
        ));
    } else {
        let text = fs::read_to_string(ctx.repo_root.join(e2e_invariants_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let invariant_count = text
            .lines()
            .filter(|line| line.trim_start().starts_with("- "))
            .count();
        if invariant_count < 5 {
            violations.push(violation(
                "OPS_E2E_INVARIANTS_CONTRACT_TOO_SMALL",
                format!(
                    "end-to-end invariants contract must define at least 5 invariants; found {invariant_count}"
                ),
                "add at least five concrete end-to-end invariants with deterministic pass criteria",
                Some(e2e_invariants_rel),
            ));
        }
    }

    let readiness_score_rel = Path::new("ops/report/generated/readiness-score.json");
    let readiness_score_schema_rel = Path::new("ops/schema/report/readiness-score.schema.json");
    if ctx.adapters.fs.exists(ctx.repo_root, readiness_score_rel)
        && ctx.adapters.fs.exists(ctx.repo_root, readiness_score_schema_rel)
    {
        let score_text = fs::read_to_string(ctx.repo_root.join(readiness_score_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let score_json: serde_json::Value =
            serde_json::from_str(&score_text).map_err(|err| CheckError::Failed(err.to_string()))?;
        if score_json.get("generated_by").and_then(|v| v.as_str()).unwrap_or("").trim().is_empty()
        {
            violations.push(violation(
                "OPS_READINESS_SCORE_GENERATOR_METADATA_MISSING",
                "readiness-score.json must include non-empty generated_by".to_string(),
                "add generated_by metadata to ops/report/generated/readiness-score.json",
                Some(readiness_score_rel),
            ));
        }
        let inputs = score_json
            .get("inputs")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        let input_keys = inputs.keys().cloned().collect::<Vec<_>>();
        let sorted_keys = {
            let mut keys = input_keys.clone();
            keys.sort();
            keys
        };
        if input_keys != sorted_keys {
            violations.push(violation(
                "OPS_READINESS_SCORE_INPUT_ORDER_NONDETERMINISTIC",
                "readiness-score.json inputs keys must be lexicographically ordered".to_string(),
                "regenerate readiness-score.json with deterministic key ordering",
                Some(readiness_score_rel),
            ));
        }

        let schema_text = fs::read_to_string(ctx.repo_root.join(readiness_score_schema_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let schema_json: serde_json::Value = serde_json::from_str(&schema_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let required_fields = schema_json
            .get("required")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(ToString::to_string))
            .collect::<BTreeSet<_>>();
        if !required_fields.contains("generated_by") {
            violations.push(violation(
                "OPS_READINESS_SCORE_SCHEMA_GENERATOR_METADATA_MISSING",
                "readiness-score.schema.json must require generated_by".to_string(),
                "add generated_by to the readiness score schema required fields",
                Some(readiness_score_schema_rel),
            ));
        }
    }

    Ok(())
}
