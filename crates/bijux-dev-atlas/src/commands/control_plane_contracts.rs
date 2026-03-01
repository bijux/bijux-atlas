pub(crate) fn run_contracts_command(quiet: bool, command: ContractsCommand) -> i32 {
    let run = (|| -> Result<(String, i32), String> {
        if let ContractsCommand::Snapshot(args) = &command {
            let repo_root = resolve_repo_root(args.repo_root.clone())?;
            let domains = all_domains(&repo_root)?;
            let (domain_name, rows, default_rel) = match args.domain {
                ContractsSnapshotDomainArg::All => (
                    "all",
                    domains
                        .iter()
                        .map(|(descriptor, registry)| {
                            contracts::registry_snapshot_with_policy(&repo_root, descriptor.name, registry)
                        })
                        .collect::<Result<Vec<_>, _>>()?
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>(),
                    PathBuf::from("artifacts/contracts/all/registry-snapshot.json"),
                ),
                ContractsSnapshotDomainArg::Root => (
                    "root",
                    contracts::registry_snapshot_with_policy(
                        &repo_root,
                        "root",
                        domain_registry(&domains, "root")?,
                    )?,
                    PathBuf::from("artifacts/contracts/root/registry-snapshot.json"),
                ),
                ContractsSnapshotDomainArg::Repo => (
                    "repo",
                    contracts::registry_snapshot_with_policy(
                        &repo_root,
                        "repo",
                        domain_registry(&domains, "repo")?,
                    )?,
                    PathBuf::from("artifacts/contracts/repo/registry-snapshot.json"),
                ),
                ContractsSnapshotDomainArg::Crates => (
                    "crates",
                    contracts::registry_snapshot_with_policy(
                        &repo_root,
                        "crates",
                        domain_registry(&domains, "crates")?,
                    )?,
                    PathBuf::from("artifacts/contracts/crates/registry-snapshot.json"),
                ),
                ContractsSnapshotDomainArg::Runtime => (
                    "runtime",
                    contracts::registry_snapshot_with_policy(
                        &repo_root,
                        "runtime",
                        domain_registry(&domains, "runtime")?,
                    )?,
                    PathBuf::from("artifacts/contracts/runtime/registry-snapshot.json"),
                ),
                ContractsSnapshotDomainArg::ControlPlane => (
                    "control-plane",
                    contracts::registry_snapshot_with_policy(
                        &repo_root,
                        "control-plane",
                        domain_registry(&domains, "control-plane")?,
                    )?,
                    PathBuf::from("artifacts/contracts/control-plane/registry-snapshot.json"),
                ),
                ContractsSnapshotDomainArg::Configs => (
                    "configs",
                    contracts::registry_snapshot_with_policy(
                        &repo_root,
                        "configs",
                        domain_registry(&domains, "configs")?,
                    )?,
                    PathBuf::from("artifacts/contracts/configs/registry-snapshot.json"),
                ),
                ContractsSnapshotDomainArg::Docs => (
                    "docs",
                    contracts::registry_snapshot_with_policy(
                        &repo_root,
                        "docs",
                        domain_registry(&domains, "docs")?,
                    )?,
                    PathBuf::from("artifacts/contracts/docs/registry-snapshot.json"),
                ),
                ContractsSnapshotDomainArg::Docker => (
                    "docker",
                    contracts::registry_snapshot_with_policy(
                        &repo_root,
                        "docker",
                        domain_registry(&domains, "docker")?,
                    )?,
                    PathBuf::from("artifacts/contracts/docker/registry-snapshot.json"),
                ),
                ContractsSnapshotDomainArg::Make => (
                    "make",
                    contracts::registry_snapshot_with_policy(
                        &repo_root,
                        "make",
                        domain_registry(&domains, "make")?,
                    )?,
                    PathBuf::from("artifacts/contracts/make/registry-snapshot.json"),
                ),
                ContractsSnapshotDomainArg::Ops => (
                    "ops",
                    contracts::registry_snapshot_with_policy(
                        &repo_root,
                        "ops",
                        domain_registry(&domains, "ops")?,
                    )?,
                    PathBuf::from("artifacts/contracts/ops/registry-snapshot.json"),
                ),
            };
            let payload = serde_json::json!({
                "schema_version": 1,
                "domain": domain_name,
                "contracts": rows.iter().map(|row| serde_json::json!({
                    "domain": row.domain,
                    "id": row.id,
                    "title": row.title,
                    "tests": row.test_ids,
                })).collect::<Vec<_>>()
            });
            let rendered = serde_json::to_string_pretty(&payload)
                .map_err(|e| format!("encode contracts snapshot failed: {e}"))?;
            let out_path = args.out.clone().unwrap_or_else(|| repo_root.join(default_rel));
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("create {} failed: {e}", parent.display()))?;
            }
            fs::write(&out_path, format!("{rendered}\n"))
                .map_err(|e| format!("write {} failed: {e}", out_path.display()))?;
            return Ok((rendered, 0));
        }

        if let ContractsCommand::SelfCheck(args) = &command {
            let repo_root = resolve_repo_root(args.repo_root.clone())?;
            let format = common_format(args);
            let mut lints = registry_lints(&repo_root)?;
            let domains = all_domains(&repo_root)?;
            let catalogs = domains
                .iter()
                .map(|(descriptor, registry)| (descriptor.name, registry.as_slice()))
                .collect::<Vec<_>>();
            lints.extend(contracts::lint_contracts(&catalogs));
            if lints.is_empty() {
                let rendered = match format {
                    ContractsFormatArg::Json => serde_json::to_string_pretty(&serde_json::json!({
                        "schema_version": 1,
                        "status": "pass",
                        "check": "contracts-self-check",
                        "lint_count": 0
                    }))
                    .map_err(|e| format!("encode contracts self-check failed: {e}"))?,
                    ContractsFormatArg::Human
                    | ContractsFormatArg::Table
                    | ContractsFormatArg::Junit
                    | ContractsFormatArg::Github => {
                        "contracts self-check: PASS".to_string()
                    }
                };
                return Ok((rendered, 0));
            }
            return Ok((render_registry_lints(&lints, format)?, 1));
        }

        let (repo_root, mut common, domain_names, contract_filter_override) = match &command {
            ContractsCommand::All(args) => (
                resolve_repo_root(args.repo_root.clone())?,
                args.clone(),
                vec![
                    "root",
                    "repo",
                    "crates",
                    "runtime",
                    "control-plane",
                    "docker",
                    "make",
                    "ops",
                    "configs",
                    "docs",
                ],
                None,
            ),
            ContractsCommand::Root(args) => (
                resolve_repo_root(args.repo_root.clone())?,
                args.clone(),
                vec!["root"],
                None,
            ),
            ContractsCommand::Repo(args) => (
                resolve_repo_root(args.repo_root.clone())?,
                args.clone(),
                vec!["repo"],
                None,
            ),
            ContractsCommand::Crates(args) => (
                resolve_repo_root(args.repo_root.clone())?,
                args.clone(),
                vec!["crates"],
                None,
            ),
            ContractsCommand::Runtime(args) => (
                resolve_repo_root(args.repo_root.clone())?,
                args.clone(),
                vec!["runtime"],
                None,
            ),
            ContractsCommand::ControlPlane(args) => (
                resolve_repo_root(args.repo_root.clone())?,
                args.clone(),
                vec!["control-plane"],
                None,
            ),
            ContractsCommand::Configs(args) => (
                resolve_repo_root(args.repo_root.clone())?,
                args.clone(),
                vec!["configs"],
                None,
            ),
            ContractsCommand::Docs(args) => (
                resolve_repo_root(args.repo_root.clone())?,
                args.clone(),
                vec!["docs"],
                None,
            ),
            ContractsCommand::Docker(args) => (
                resolve_repo_root(args.common.repo_root.clone())?,
                args.common.clone(),
                vec!["docker"],
                None,
            ),
            ContractsCommand::Make(args) => (
                resolve_repo_root(args.common.repo_root.clone())?,
                args.common.clone(),
                vec!["make"],
                None,
            ),
            ContractsCommand::Ops(args) => (
                resolve_repo_root(args.common.repo_root.clone())?,
                args.common.clone(),
                vec!["ops"],
                args.domain.map(ops_domain_filter),
            ),
            ContractsCommand::SelfCheck(_) => unreachable!("handled above"),
            ContractsCommand::Snapshot(_) => unreachable!("handled above"),
        };
        apply_lane_policy(&mut common);
        apply_ci_policy(&mut common);
        validate_selection_patterns(&common).map_err(|err| format!("usage: {err}"))?;

        let format = common_format(&common);
        let lints = registry_lints(&repo_root)?;
        if !lints.is_empty() {
            return Ok((render_registry_lints(&lints, format)?, 2));
        }
        require_skip_policy(&common.skip_contracts)?;

        let changed_paths = if common.changed_only {
            changed_paths_since_merge_base(&repo_root)
        } else {
            None
        };
        let mut selected_domains = all_domains(&repo_root)?
            .into_iter()
            .filter(|(descriptor, _)| domain_names.iter().any(|name| descriptor.name == *name))
            .filter(|(descriptor, _)| {
                common.groups.is_empty()
                    || common
                        .groups
                        .iter()
                        .any(|name| descriptor.name.eq_ignore_ascii_case(name))
            })
            .filter_map(|(descriptor, registry)| {
                let reason = if common.changed_only {
                    match &changed_paths {
                        Some(paths) => domain_change_reason(descriptor.name, paths)?,
                        None => "changed-only merge-base unavailable; selected by fallback".to_string(),
                    }
                } else {
                    "selected by requested contracts domain".to_string()
                };
                Some((descriptor, registry, reason))
            })
            .collect::<Vec<_>>();
        if common.changed_only && selected_domains.is_empty() {
            selected_domains = all_domains(&repo_root)?
                .into_iter()
                .filter(|(descriptor, _)| domain_names.iter().any(|name| descriptor.name == *name))
                .filter(|(descriptor, _)| {
                    common.groups.is_empty()
                        || common
                            .groups
                            .iter()
                            .any(|name| descriptor.name.eq_ignore_ascii_case(name))
                })
                .map(|(descriptor, registry)| {
                    (
                        descriptor,
                        registry,
                        "changed-only fallback: no matching changed paths for requested domains"
                            .to_string(),
                    )
                })
                .collect();
        }
        let catalogs = selected_domains
            .iter()
            .map(|(descriptor, registry, _)| (descriptor.name, registry.as_slice()))
            .collect::<Vec<_>>();
        let derived_lints = contracts::lint_contracts(&catalogs);
        if !derived_lints.is_empty() {
            return Ok((render_registry_lints(&derived_lints, format)?, 2));
        }
        forbid_skip_required(&repo_root, &selected_domains, &common, &contract_filter_override)?;
        write_required_contract_artifact(&repo_root, &all_domains(&repo_root)?)?;

        if common.list || common.list_tests {
            let list_domains = selected_domains
                .iter()
                .map(|(descriptor, registry, _)| (*descriptor, registry.as_slice()))
                .collect::<Vec<_>>();
            return Ok((render_list(&repo_root, &list_domains, common.list_tests, format)?, 0));
        }

        if let Some(test_id) = &common.explain_test {
            for (descriptor, registry, _) in &selected_domains {
                for contract in registry {
                    if let Some(test) = contract
                        .tests
                        .iter()
                        .find(|test| test.id.0.eq_ignore_ascii_case(test_id))
                    {
                        return Ok((explain_test(descriptor.name, contract, test, format)?, 0));
                    }
                }
            }
            return usage_error(format!("unknown contract test id `{test_id}`"));
        }

        if let Some(contract_id) = &common.explain {
            for (descriptor, registry, _) in &selected_domains {
                if let Some(contract) = registry
                    .iter()
                    .find(|entry| entry.id.0.eq_ignore_ascii_case(contract_id))
                {
                    let mapped_gates = if descriptor.name == "ops" {
                        ops_mapped_gates(&repo_root, &contract.id.0)
                    } else {
                        Vec::new()
                    };
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "domain": descriptor.name,
                        "contract_id": contract.id.0,
                        "title": contract.title,
                        "tests": contract.tests.iter().map(|case| serde_json::json!({
                            "test_id": case.id.0,
                            "title": case.title
                        })).collect::<Vec<_>>(),
                        "mapped_gate": (descriptor.gate_fn)(&contract.id.0),
                        "mapped_gates": mapped_gates,
                        "mapped_command": (descriptor.gate_fn)(&contract.id.0),
                        "explain": (descriptor.explain_fn)(&contract.id.0)
                    });
                    let rendered = match format {
                        ContractsFormatArg::Json => serde_json::to_string_pretty(&payload)
                            .map_err(|e| format!("encode contracts explain failed: {e}"))?,
                        ContractsFormatArg::Human
                        | ContractsFormatArg::Table
                        | ContractsFormatArg::Junit
                        | ContractsFormatArg::Github => {
                            let mut out = String::new();
                            out.push_str(&format!("{} {}\n", contract.id.0, contract.title));
                            out.push_str("Tests:\n");
                            for case in &contract.tests {
                                out.push_str(&format!("- {}: {}\n", case.id.0, case.title));
                            }
                            out.push_str("\nIntent:\n");
                            out.push_str(&(descriptor.explain_fn)(&contract.id.0));
                            out.push_str("\n\nMapped gate:\n");
                            out.push_str((descriptor.gate_fn)(&contract.id.0));
                            out.push('\n');
                            if !mapped_gates.is_empty() {
                                out.push_str("Mapped gates:\n");
                                for gate in mapped_gates {
                                    out.push_str(&format!("- {gate}\n"));
                                }
                            }
                            out
                        }
                    };
                    return Ok((rendered, 0));
                }
            }
            return usage_error(format!("unknown contract id `{contract_id}`"));
        }

        if common.mode == ContractsModeArg::Effect {
            let contract_filter = contract_filter_override
                .clone()
                .or_else(|| common.filter_contract.clone());
            let mut requires_subprocess = false;
            let mut requires_network = false;
            let mut requires_k8s = false;
            let mut requires_fs_write = false;
            let mut requires_docker_daemon = false;
            for (_, registry, _) in &selected_domains {
                let required = contracts::required_effects_for_selection(
                    registry,
                    contracts::Mode::Effect,
                    contracts::SelectionFilters {
                        contract_filter: contract_filter.as_deref(),
                        test_filter: common.filter_test.as_deref(),
                        only_contracts: &common.only_contracts,
                        only_tests: &common.only_tests,
                        skip_contracts: &common.skip_contracts,
                        tags: &common.tags,
                    },
                );
                requires_subprocess |= required.allow_subprocess;
                requires_network |= required.allow_network;
                requires_k8s |= required.allow_k8s;
                requires_fs_write |= required.allow_fs_write;
                requires_docker_daemon |= required.allow_docker_daemon;
            }
            let mut missing = Vec::new();
            if requires_subprocess && !common.allow_subprocess {
                missing.push("--allow-subprocess");
            }
            if requires_network && !common.allow_network {
                missing.push("--allow-network");
            }
            if requires_k8s && !common.allow_k8s {
                missing.push("--allow-k8s");
            }
            if requires_fs_write && !common.allow_fs_write {
                missing.push("--allow-fs-write");
            }
            if requires_docker_daemon && !common.allow_docker_daemon {
                missing.push("--allow-docker-daemon");
            }
            if !missing.is_empty() {
                return usage_error(format!(
                    "effect mode requires {} for the selected contracts",
                    missing.join(", ")
                ));
            }
        }

        let mut reports = Vec::new();
        for (descriptor, _, _) in &selected_domains {
            let mut run_common = common.clone();
            if domain_names.len() > 1 {
                if let Some(root) = &common.artifacts_root {
                    run_common.artifacts_root = Some(root.join(descriptor.name));
                }
            }
            reports.push(run_one(
                descriptor,
                &repo_root,
                &run_common,
                contract_filter_override.clone().or_else(|| common.filter_contract.clone()),
            )?);
        }

        let selection_header = if matches!(
            format,
            ContractsFormatArg::Human | ContractsFormatArg::Table | ContractsFormatArg::Github
        ) {
            let mut lines = Vec::new();
            if common.changed_only {
                let base = std::env::var("CONTRACTS_CHANGED_BASE")
                    .ok()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| "HEAD".to_string());
                lines.push(format!(
                    "Changed-only selection base: merge-base(HEAD, {base})"
                ));
                if matches!(changed_paths, Some(ref paths) if paths.is_empty()) {
                    lines.push("Changed-only selection note: no merge-base diff paths detected; no domains selected".to_string());
                } else if changed_paths.is_none() {
                    lines.push("Changed-only selection note: merge-base diff could not be resolved; selecting requested domains".to_string());
                }
            }
            if !selected_domains.is_empty() {
                lines.push("Selected domains:".to_string());
                for (descriptor, _, reason) in &selected_domains {
                    lines.push(format!("- {} ({reason})", descriptor.name));
                }
            }
            if lines.is_empty() {
                String::new()
            } else {
                format!("{}\n\n", lines.join("\n"))
            }
        } else {
            String::new()
        };

        let rendered = match format {
            ContractsFormatArg::Human => {
                if reports.len() == 1 {
                    format!("{selection_header}{}", contracts::to_pretty(&reports[0]))
                } else {
                    format!("{selection_header}{}", contracts::to_pretty_all(&reports))
                }
            }
            ContractsFormatArg::Table => {
                if reports.len() == 1 {
                    format!("{selection_header}{}", contracts::to_table(&reports[0]))
                } else {
                    format!("{selection_header}{}", contracts::to_table_all(&reports))
                }
            }
            ContractsFormatArg::Json => serde_json::to_string_pretty(&if reports.len() == 1 {
                contracts::to_json(&reports[0])
            } else {
                contracts::to_json_all(&reports)
            })
            .map_err(|e| format!("encode contracts report failed: {e}"))?,
            ContractsFormatArg::Junit => {
                if reports.len() == 1 {
                    contracts::to_junit(&reports[0])?
                } else {
                    contracts::to_junit_all(&reports)?
                }
            }
            ContractsFormatArg::Github => {
                format!("{selection_header}{}", contracts::to_github(&reports))
            }
        };

        if reports.len() > 1 {
            if let Some(root) = &common.artifacts_root {
                fs::create_dir_all(root)
                    .map_err(|e| format!("create {} failed: {e}", root.display()))?;
                let coverage = serde_json::json!({
                    "schema_version": 1,
                    "groups": reports.iter().map(|report| {
                        let coverage = contracts::coverage_report(report);
                        serde_json::json!({
                            "group": coverage.group,
                            "contracts": coverage.contracts,
                            "tests": coverage.tests,
                            "pass": coverage.pass,
                            "fail": coverage.fail,
                            "skip": coverage.skip,
                            "error": coverage.error,
                        })
                    }).collect::<Vec<_>>()
                });
                let coverage_path = root.join("all.coverage.json");
                fs::write(
                    &coverage_path,
                    serde_json::to_string_pretty(&coverage)
                        .map_err(|e| format!("encode contracts all coverage failed: {e}"))?,
                )
                .map_err(|e| format!("write {} failed: {e}", coverage_path.display()))?;
                let summary = serde_json::json!({
                    "schema_version": 1,
                    "kind": "contracts-summary",
                    "groups": reports.iter().map(|report| serde_json::json!({
                        "group": report.domain,
                        "contracts": report.total_contracts(),
                        "tests": report.total_tests(),
                        "pass": report.pass_count(),
                        "fail": report.fail_count(),
                        "skip": report.skip_count(),
                        "error": report.error_count(),
                        "exit_code": report.exit_code(),
                    })).collect::<Vec<_>>()
                });
                let summary_path = root.join("contracts-summary.json");
                fs::write(
                    &summary_path,
                    serde_json::to_string_pretty(&summary)
                        .map_err(|e| format!("encode contracts summary failed: {e}"))?,
                )
                .map_err(|e| format!("write {} failed: {e}", summary_path.display()))?;

                let unified_json_path = root.join("unified.json");
                fs::write(
                    &unified_json_path,
                    serde_json::to_string_pretty(&contracts::to_json_all(&reports))
                        .map_err(|e| format!("encode contracts unified json failed: {e}"))?,
                )
                .map_err(|e| format!("write {} failed: {e}", unified_json_path.display()))?;

                let unified_md_path = root.join("unified.md");
                fs::write(&unified_md_path, contracts::to_pretty_all(&reports))
                    .map_err(|e| format!("write {} failed: {e}", unified_md_path.display()))?;
            }
        }

        let json_rendered = serde_json::to_string_pretty(&if reports.len() == 1 {
            contracts::to_json(&reports[0])
        } else {
            contracts::to_json_all(&reports)
        })
        .map_err(|e| format!("encode contracts json report failed: {e}"))?;
        let junit_rendered = if reports.len() == 1 {
            contracts::to_junit(&reports[0])?
        } else {
            contracts::to_junit_all(&reports)?
        };
        write_optional(&common.json_out, &json_rendered)?;
        write_optional(&common.junit_out, &junit_rendered)?;

        Ok((
            rendered,
            reports
                .iter()
                .map(contracts::RunReport::exit_code)
                .max()
                .unwrap_or(0),
        ))
    })();

    match run {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                if code == 0 {
                    let _ = writeln!(io::stdout(), "{rendered}");
                } else {
                    let _ = writeln!(io::stderr(), "{rendered}");
                }
            }
            code
        }
        Err(err) => {
            let (message, code) = if let Some(detail) = err.strip_prefix("usage: ") {
                (detail.to_string(), 2)
            } else {
                (err, 3)
            };
            let _ = writeln!(io::stderr(), "bijux-dev-atlas contracts failed: {message}");
            code
        }
    }
}

// Contracts CLI guardrails:
// changed-only selection must use git merge-base and git diff --name-only.
const _CHANGED_ONLY_SELECTION_SENTINEL: &[&str] = &[
    "merge-base", "HEAD", "diff", "--name-only",
];
