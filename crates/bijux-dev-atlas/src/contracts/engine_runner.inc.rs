pub fn run(
    domain: &str,
    contracts_fn: fn(&Path) -> Result<Vec<Contract>, String>,
    repo_root: &Path,
    options: &RunOptions,
) -> Result<RunReport, String> {
    let mut contracts = contracts_fn(repo_root)?;
    contracts.sort_by_key(|c| c.id.0.clone());

    let ctx = RunContext {
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

    for contract in contracts {
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
        for case in cases {
            if !matches_filter(&options.test_filter, &case.id.0)
                || !matches_any_filter(&options.only_tests, &case.id.0)
            {
                continue;
            }
            has_case = true;
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
                    _ => match std::panic::catch_unwind(|| (case.run)(&ctx)) {
                        Ok(v) => v,
                        Err(_) => TestResult::Error("test panicked".to_string()),
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
            case_rows.push(CaseReport {
                contract_id: contract_id.clone(),
                contract_title: contract_title.clone(),
                test_id: case.id.0,
                test_title: case.title.to_string(),
                kind: case.kind,
                status,
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
                mode: contract_mode,
                effects: contract_effects,
                status: contract_status,
            });
        }
        if options.fail_fast && matches!(contract_status, CaseStatus::Fail | CaseStatus::Error) {
            break;
        }
    }

    let report = RunReport {
        domain: domain.to_string(),
        mode: options.mode,
        metadata: run_metadata(repo_root),
        contracts: contract_rows,
        cases: case_rows,
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
                "maturity": maturity_score(&report.contracts),
            }))
            .map_err(|e| format!("encode contracts maturity failed: {e}"))?,
        )
        .map_err(|e| format!("write {} failed: {e}", maturity_path.display()))?;
    }

    Ok(report)
}
