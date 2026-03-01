use std::time::Instant;

fn panic_payload_to_string(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(text) = payload.downcast_ref::<&'static str>() {
        (*text).to_string()
    } else if let Some(text) = payload.downcast_ref::<String>() {
        text.clone()
    } else {
        "non-string panic payload".to_string()
    }
}

pub fn run(
    domain: &str,
    contracts_fn: fn(&Path) -> Result<Vec<Contract>, String>,
    repo_root: &Path,
    options: &RunOptions,
) -> Result<RunReport, String> {
    let run_started = Instant::now();
    let mut contracts = contracts_fn(repo_root)?;
    contracts.sort_by_key(|c| c.id.0.clone());
    let required_map = required_contract_map(repo_root)?;

    let base_ctx = RunContext {
        repo_root: repo_root.to_path_buf(),
        artifacts_root: options.artifacts_root.clone(),
        mode: options.mode,
        allow_subprocess: options.allow_subprocess,
        allow_network: options.allow_network,
        allow_k8s: options.allow_k8s,
        allow_fs_write: options.allow_fs_write,
        allow_docker_daemon: options.allow_docker_daemon,
        skip_missing_tools: options.skip_missing_tools,
        timeout_seconds: options.timeout_seconds,
    };

    let mut contract_rows = Vec::new();
    let mut case_rows = Vec::new();
    let mut panic_rows = Vec::new();

    for contract in contracts {
        let required_lanes = contract_required_lanes(&required_map, domain, &contract.id.0);
        let is_required = required_lanes.contains(&options.lane);
        if options.required_only && !is_required {
            continue;
        }
        if !matches_filter(&options.contract_filter, &contract.id.0)
            || !matches_any_filter(&options.only_contracts, &contract.id.0)
            || matches_skip_filter(&options.skip_contracts, &contract.id.0)
            || !matches_tags(&options.tags, &contract)
        {
            continue;
        }
        let contract_mode = contract_mode(&contract);
        let contract_effects = contract_effects(&contract);
        let contract_id = contract.id.0.clone();
        let contract_title = contract.title.to_string();
        let mut cases = contract.tests;
        cases.sort_by_key(|t| t.id.0.clone());
        let mut contract_status = CaseStatus::Pass;
        let mut has_case = false;
        let contract_started = Instant::now();
        for case in cases {
            if !matches_filter(&options.test_filter, &case.id.0)
                || !matches_any_filter(&options.only_tests, &case.id.0)
            {
                continue;
            }
            has_case = true;
            let case_started = Instant::now();
            let ctx = RunContext {
                skip_missing_tools: if is_required {
                    false
                } else {
                    base_ctx.skip_missing_tools
                },
                ..base_ctx.clone()
            };
            let result = if options.list_only {
                TestResult::Skip("list-only".to_string())
            } else {
                match (options.mode, case.kind) {
                    (Mode::Static, TestKind::Subprocess | TestKind::Network) => {
                        TestResult::Skip("effect-only test".to_string())
                    }
                    (Mode::Effect, TestKind::Subprocess) if !options.allow_subprocess => {
                        TestResult::Error("requires --allow-subprocess".to_string())
                    }
                    (Mode::Effect, TestKind::Network) if !options.allow_network => {
                        TestResult::Error("requires --allow-network".to_string())
                    }
                    _ => match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        (case.run)(&ctx)
                    })) {
                        Ok(v) => v,
                        Err(payload) => {
                            let payload_text = panic_payload_to_string(payload.as_ref());
                            let backtrace = std::backtrace::Backtrace::force_capture().to_string();
                            panic_rows.push(PanicRecord {
                                domain: domain.to_string(),
                                contract_id: contract_id.clone(),
                                test_id: case.id.0.clone(),
                                payload: payload_text.clone(),
                                backtrace: backtrace.clone(),
                            });
                            TestResult::Error(format!("test panicked: {payload_text}"))
                        }
                    },
                }
            };
            let status = case_status_from_result(&result);
            contract_status = worst_status(contract_status, status);
            let (violations, note) = match result {
                TestResult::Pass => (Vec::new(), None),
                TestResult::Fail(rows) => (rows, None),
                TestResult::Skip(reason) => (Vec::new(), Some(reason)),
                TestResult::Error(err) => (Vec::new(), Some(err)),
            };
            let mut violations = violations;
            violations.sort_by(|a, b| {
                a.contract_id
                    .cmp(&b.contract_id)
                    .then(a.file.cmp(&b.file))
                    .then(a.line.cmp(&b.line))
                    .then(a.message.cmp(&b.message))
            });
            case_rows.push(CaseReport {
                contract_id: contract_id.clone(),
                contract_title: contract_title.clone(),
                required: is_required,
                lanes: required_lanes.clone(),
                test_id: case.id.0,
                test_title: case.title.to_string(),
                kind: case.kind,
                status,
                duration_ms: case_started.elapsed().as_millis() as u64,
                violations,
                note,
            });
            if options.fail_fast && matches!(status, CaseStatus::Fail | CaseStatus::Error) {
                break;
            }
        }
        if has_case {
            contract_rows.push(ContractSummary {
                id: contract_id,
                title: contract_title,
                required: is_required,
                lanes: required_lanes.clone(),
                mode: contract_mode,
                effects: contract_effects,
                status: contract_status,
                duration_ms: contract_started.elapsed().as_millis() as u64,
            });
        }
        if options.fail_fast && matches!(contract_status, CaseStatus::Fail | CaseStatus::Error) {
            break;
        }
    }

    let report = RunReport {
        domain: domain.to_string(),
        lane: options.lane,
        mode: options.mode,
        metadata: run_metadata(
            repo_root,
            options.ci,
            options.color_enabled,
            options.run_id.as_deref(),
        ),
        contracts: contract_rows,
        cases: case_rows,
        panics: panic_rows,
        duration_ms: run_started.elapsed().as_millis() as u64,
    };

    if let Some(root) = &options.artifacts_root {
        let out_dir = root.clone();
        std::fs::create_dir_all(&out_dir)
            .map_err(|e| format!("create contracts artifact dir failed: {e}"))?;
        let json_path = out_dir.join(format!("{domain}.json"));
        std::fs::write(
            &json_path,
            serde_json::to_string_pretty(&to_json(&report))
                .map_err(|e| format!("encode contracts report failed: {e}"))?,
        )
        .map_err(|e| format!("write {} failed: {e}", json_path.display()))?;
        let inventory_path = out_dir.join(format!("{domain}.inventory.json"));
        std::fs::write(
            &inventory_path,
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "domain": domain,
                "contracts": report.contracts.iter().map(|contract| serde_json::json!({
                    "id": contract.id,
                    "title": contract.title,
                    "required": contract.required,
                    "lanes": contract.lanes.iter().map(|lane| lane.as_str()).collect::<Vec<_>>(),
                    "mode": contract.mode.as_str(),
                    "effects": contract.effects.iter().map(|effect| effect.as_str()).collect::<Vec<_>>(),
                })).collect::<Vec<_>>()
            }))
            .map_err(|e| format!("encode contracts inventory failed: {e}"))?,
        )
        .map_err(|e| format!("write {} failed: {e}", inventory_path.display()))?;
        let maturity_path = out_dir.join(format!("{domain}.maturity.json"));
        std::fs::write(
            &maturity_path,
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "domain": domain,
                "lane": report.lane.as_str(),
                "maturity": maturity_score(&report.contracts),
            }))
            .map_err(|e| format!("encode contracts maturity failed: {e}"))?,
        )
        .map_err(|e| format!("write {} failed: {e}", maturity_path.display()))?;
        let table_path = out_dir.join("table.txt");
        let mut table = String::new();
        table.push_str(&format!("Contracts: {} (mode={})\n", report.domain, report.mode));
        for contract in &report.contracts {
            table.push_str(&format!(
                "{}\t{}\t{}\n",
                contract.id,
                contract.status.as_str(),
                contract.title
            ));
        }
        std::fs::write(&table_path, table)
            .map_err(|e| format!("write {} failed: {e}", table_path.display()))?;
        let status_path = out_dir.join("status.json");
        std::fs::write(
            &status_path,
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "domain": domain,
                "lane": report.lane.as_str(),
                "mode": report.mode.to_string(),
                "run_id": report.metadata.run_id,
                "contracts": report.contracts.iter().map(|contract| serde_json::json!({
                    "id": contract.id,
                    "required": contract.required,
                    "lanes": contract.lanes.iter().map(|lane| lane.as_str()).collect::<Vec<_>>(),
                    "status": contract.status.as_str(),
                    "title": contract.title,
                })).collect::<Vec<_>>()
            }))
            .map_err(|e| format!("encode contracts status failed: {e}"))?,
        )
        .map_err(|e| format!("write {} failed: {e}", status_path.display()))?;
        let meta_path = out_dir.join("meta.json");
        std::fs::write(
            &meta_path,
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "gate": "contracts",
                "domain": domain,
                "lane": report.lane.as_str(),
                "mode": report.mode.to_string(),
                "run_id": report.metadata.run_id,
                "commit_sha": report.metadata.commit_sha,
                "dirty_tree": report.metadata.dirty_tree,
                "ci": report.metadata.ci,
                "success": report.exit_code() == 0,
                "duration_ms": report.duration_ms,
            }))
            .map_err(|e| format!("encode contracts meta failed: {e}"))?,
        )
        .map_err(|e| format!("write {} failed: {e}", meta_path.display()))?;
        let summary_path = out_dir.join("summary.json");
        std::fs::write(
            &summary_path,
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "gate": "contracts",
                "domain": domain,
                "contracts": report.total_contracts(),
                "tests": report.total_tests(),
                "pass": report.pass_count(),
                "fail": report.fail_count(),
                "skip": report.skip_count(),
                "error": report.error_count(),
                "panic_count": report.panics.len(),
                "success": report.exit_code() == 0,
            }))
            .map_err(|e| format!("encode contracts summary failed: {e}"))?,
        )
        .map_err(|e| format!("write {} failed: {e}", summary_path.display()))?;
        let panics_path = out_dir.join("panics.json");
        std::fs::write(
            &panics_path,
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "domain": domain,
                "run_id": report.metadata.run_id,
                "panics": report.panics.iter().map(|panic| serde_json::json!({
                    "domain": panic.domain,
                    "contract_id": panic.contract_id,
                    "test_id": panic.test_id,
                    "payload": panic.payload,
                    "backtrace": panic.backtrace,
                })).collect::<Vec<_>>()
            }))
            .map_err(|e| format!("encode contracts panic report failed: {e}"))?,
        )
        .map_err(|e| format!("write {} failed: {e}", panics_path.display()))?;
        let coverage = coverage_report(&report);
        let coverage_path = out_dir.join(format!("{domain}.coverage.json"));
        std::fs::write(
            &coverage_path,
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "group": coverage.group,
                "contracts": coverage.contracts,
                "tests": coverage.tests,
                "pass": coverage.pass,
                "fail": coverage.fail,
                "skip": coverage.skip,
                "error": coverage.error,
                "coverage_pct": if coverage.tests == 0 { 100 } else { ((coverage.pass as f64 / coverage.tests as f64) * 100.0).round() as u64 }
            }))
            .map_err(|e| format!("encode contracts coverage failed: {e}"))?,
        )
        .map_err(|e| format!("write {} failed: {e}", coverage_path.display()))?;
        let touched_paths_path = out_dir.join(format!("{domain}.touched-paths.json"));
        std::fs::write(
            &touched_paths_path,
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "group": domain,
                "contracts": report.contracts.iter().map(|contract| {
                    let mut paths = report
                        .cases
                        .iter()
                        .filter(|case| case.contract_id == contract.id)
                        .flat_map(|case| case.violations.iter().filter_map(|violation| violation.file.clone()))
                        .collect::<Vec<_>>();
                    paths.sort();
                    paths.dedup();
                    serde_json::json!({
                        "contract_id": contract.id,
                        "paths": paths
                    })
                }).collect::<Vec<_>>()
            }))
            .map_err(|e| format!("encode contracts touched paths failed: {e}"))?,
        )
        .map_err(|e| format!("write {} failed: {e}", touched_paths_path.display()))?;
        let dependency_graph_path = out_dir.join(format!("{domain}.dependency-graph.json"));
        std::fs::write(
            &dependency_graph_path,
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "group": domain,
                "nodes": report.contracts.iter().map(|contract| serde_json::json!({
                    "id": contract.id,
                    "title": contract.title
                })).collect::<Vec<_>>(),
                "edges": Vec::<serde_json::Value>::new()
            }))
            .map_err(|e| format!("encode contracts dependency graph failed: {e}"))?,
        )
        .map_err(|e| format!("write {} failed: {e}", dependency_graph_path.display()))?;
    }

    Ok(report)
}
