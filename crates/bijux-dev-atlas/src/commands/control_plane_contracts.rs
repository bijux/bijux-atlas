pub(crate) fn run_contracts_command(quiet: bool, command: ContractsCommand) -> i32 {
    fn usage_error(message: impl Into<String>) -> Result<(String, i32), String> {
        Err(format!("usage: {}", message.into()))
    }

    fn require_skip_policy(skip_contracts: &[String]) -> Result<(), String> {
        if std::env::var_os("CI").is_some()
            && !skip_contracts.is_empty()
            && std::env::var_os("CONTRACTS_ALLOW_SKIP").is_none()
        {
            return Err(
                "CI contracts runs forbid --skip unless CONTRACTS_ALLOW_SKIP is set".to_string(),
            );
        }
        Ok(())
    }

    fn ops_domain_filter(domain: ContractsOpsDomainArg) -> String {
        match domain {
            ContractsOpsDomainArg::Root => "OPS-ROOT-*".to_string(),
            ContractsOpsDomainArg::Datasets => "OPS-DATASETS-*".to_string(),
            ContractsOpsDomainArg::E2e => "OPS-E2E-*".to_string(),
            ContractsOpsDomainArg::Env => "OPS-ENV-*".to_string(),
            ContractsOpsDomainArg::Inventory => "OPS-INV-*".to_string(),
            ContractsOpsDomainArg::K8s => "OPS-K8S-*".to_string(),
            ContractsOpsDomainArg::Load => "OPS-LOAD-*".to_string(),
            ContractsOpsDomainArg::Observe => "OPS-OBS-*".to_string(),
            ContractsOpsDomainArg::Report => "OPS-REPORT-*".to_string(),
            ContractsOpsDomainArg::Schema => "OPS-SCHEMA-*".to_string(),
            ContractsOpsDomainArg::Stack => "OPS-STACK-*".to_string(),
        }
    }

    fn common_format(common: &ContractsCommonArgs) -> ContractsFormatArg {
        if common.json {
            ContractsFormatArg::Json
        } else {
            common.format
        }
    }

    fn apply_lane_policy(common: &mut ContractsCommonArgs) -> Result<(), String> {
        match common.lane {
            ContractsLaneArg::Local => {}
            ContractsLaneArg::Dev => {
                common.mode = ContractsModeArg::Effect;
                common.allow_subprocess = true;
                common.allow_fs_write = true;
            }
            ContractsLaneArg::Ci => {
                common.mode = ContractsModeArg::Effect;
                common.profile = ContractsProfileArg::Ci;
                common.allow_subprocess = true;
                common.allow_network = true;
                common.allow_k8s = true;
                common.allow_fs_write = true;
                common.allow_docker_daemon = true;
            }
        }
        if common.mode == ContractsModeArg::Effect
            && common.deny_effects
            && common.lane == ContractsLaneArg::Local
        {
            return Err(
                "effect execution is denied by default; use --lane dev or --lane ci, or pass --deny-effects=false for manual allow flags".to_string(),
            );
        }
        Ok(())
    }

    fn write_optional(path: &Option<PathBuf>, rendered: &str) -> Result<(), String> {
        if let Some(path) = path {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("create {} failed: {e}", parent.display()))?;
            }
            fs::write(path, rendered).map_err(|e| format!("write {} failed: {e}", path.display()))?;
        }
        Ok(())
    }

    fn ops_mapped_gates(repo_root: &Path, contract_id: &str) -> Vec<String> {
        let path = repo_root.join("ops/inventory/contract-gate-map.json");
        let Ok(text) = fs::read_to_string(path) else {
            return Vec::new();
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
            return Vec::new();
        };
        json.get("mappings")
            .and_then(|v| v.as_array())
            .and_then(|rows| {
                rows.iter().find(|item| {
                    item.get("contract_id")
                        .and_then(|v| v.as_str())
                        .is_some_and(|value| value.eq_ignore_ascii_case(contract_id))
                })
            })
            .and_then(|item| item.get("gate_ids"))
            .and_then(|v| v.as_array())
            .map(|gate_ids| {
                gate_ids
                    .iter()
                    .filter_map(|v| v.as_str().map(ToOwned::to_owned))
                    .collect()
            })
            .unwrap_or_default()
    }

    #[derive(Clone, Copy)]
    struct DomainDescriptor {
        name: &'static str,
        contracts_fn: fn(&Path) -> Result<Vec<contracts::Contract>, String>,
        explain_fn: fn(&str) -> String,
        gate_fn: fn(&str) -> &'static str,
    }

    fn domain_descriptor(name: &str) -> Option<DomainDescriptor> {
        match name {
            "root" => Some(DomainDescriptor {
                name: "root",
                contracts_fn: contracts::root::contracts,
                explain_fn: contracts::root::contract_explain,
                gate_fn: contracts::root::contract_gate_command,
            }),
            "docker" => Some(DomainDescriptor {
                name: "docker",
                contracts_fn: contracts::docker::contracts,
                explain_fn: contracts::docker::contract_explain,
                gate_fn: |_id| "bijux dev atlas contracts docker --mode static",
            }),
            "make" => Some(DomainDescriptor {
                name: "make",
                contracts_fn: contracts::make::contracts,
                explain_fn: contracts::make::contract_explain,
                gate_fn: contracts::make::contract_gate_command,
            }),
            "ops" => Some(DomainDescriptor {
                name: "ops",
                contracts_fn: contracts::ops::contracts,
                explain_fn: |id| contracts::ops::contract_explain(id).to_string(),
                gate_fn: contracts::ops::contract_gate_command,
            }),
            "configs" => Some(DomainDescriptor {
                name: "configs",
                contracts_fn: contracts::configs::contracts,
                explain_fn: contracts::configs::contract_explain,
                gate_fn: contracts::configs::contract_gate_command,
            }),
            "docs" => Some(DomainDescriptor {
                name: "docs",
                contracts_fn: contracts::docs::contracts,
                explain_fn: contracts::docs::contract_explain,
                gate_fn: contracts::docs::contract_gate_command,
            }),
            _ => None,
        }
    }

    fn all_domains(repo_root: &Path) -> Result<Vec<(DomainDescriptor, Vec<contracts::Contract>)>, String> {
        let mut out = Vec::new();
        for name in ["root", "docker", "make", "ops", "configs", "docs"] {
            let descriptor = domain_descriptor(name)
                .ok_or_else(|| format!("internal contracts domain registry is missing `{name}`"))?;
            out.push((descriptor, (descriptor.contracts_fn)(repo_root)?));
        }
        Ok(out)
    }

    fn domain_registry<'a>(
        domains: &'a [(DomainDescriptor, Vec<contracts::Contract>)],
        name: &str,
    ) -> Result<&'a Vec<contracts::Contract>, String> {
        domains
            .iter()
            .find(|(descriptor, _)| descriptor.name == name)
            .map(|(_, registry)| registry)
            .ok_or_else(|| format!("internal contracts domain registry is missing `{name}`"))
    }

    fn registry_lints(repo_root: &Path) -> Result<Vec<contracts::RegistryLint>, String> {
        let mut rows = Vec::new();
        for (descriptor, registry) in all_domains(repo_root)? {
            rows.extend(contracts::registry_snapshot(descriptor.name, &registry));
        }
        Ok(contracts::lint_registry_rows(&rows))
    }

    fn render_registry_lints(
        lints: &[contracts::RegistryLint],
        format: ContractsFormatArg,
    ) -> Result<String, String> {
        if lints.is_empty() {
            return Ok(String::new());
        }
        match format {
            ContractsFormatArg::Json => serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "status": "invalid",
                "lints": lints.iter().map(|lint| serde_json::json!({
                    "code": lint.code,
                    "message": lint.message
                })).collect::<Vec<_>>()
            }))
            .map_err(|e| format!("encode contracts lint report failed: {e}")),
            ContractsFormatArg::Human
            | ContractsFormatArg::Table
            | ContractsFormatArg::Junit
            | ContractsFormatArg::Github => Ok(lints
                .iter()
                .map(|lint| format!("{}: {}", lint.code, lint.message))
                .collect::<Vec<_>>()
                .join("\n")),
        }
    }

    fn render_list(
        domains: &[(DomainDescriptor, Vec<contracts::Contract>)],
        include_tests: bool,
        format: ContractsFormatArg,
    ) -> Result<String, String> {
        let mut rows = Vec::new();
        for (descriptor, registry) in domains {
            rows.extend(contracts::registry_snapshot(descriptor.name, registry));
        }
        rows.sort_by(|a, b| a.domain.cmp(&b.domain).then(a.id.cmp(&b.id)));
        match format {
            ContractsFormatArg::Json => serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "contracts": rows.iter().map(|row| serde_json::json!({
                    "domain": row.domain,
                    "id": row.id,
                    "severity": row.severity,
                    "title": row.title,
                    "tests": row.test_ids.iter().map(|test_id| serde_json::json!({
                        "test_id": test_id
                    })).collect::<Vec<_>>()
                })).collect::<Vec<_>>()
            }))
            .map_err(|e| format!("encode contracts list failed: {e}")),
            ContractsFormatArg::Human
            | ContractsFormatArg::Table
            | ContractsFormatArg::Junit
            | ContractsFormatArg::Github => {
                let mut out = String::new();
                out.push_str("GROUP    CONTRACT ID        SEVERITY TITLE\n");
                for row in rows {
                    out.push_str(&format!(
                        "{:<8} {:<18} {:<8} {}\n",
                        row.domain, row.id, row.severity, row.title
                    ));
                    if include_tests {
                        for test_id in row.test_ids {
                            out.push_str(&format!("         - {}\n", test_id));
                        }
                    }
                }
                Ok(out)
            }
        }
    }

    fn domain_has_changes(repo_root: &Path, name: &str) -> bool {
        let repo_display = repo_root.display().to_string();
        let output = std::process::Command::new("git")
            .args(["-C", &repo_display, "status", "--porcelain"])
            .output();
        let Ok(output) = output else {
            return true;
        };
        if !output.status.success() {
            return true;
        }
        let changed = String::from_utf8_lossy(&output.stdout);
        let prefix = match name {
            "docs" => Some("docs/"),
            "root" => None,
            _ => return true,
        };
        for line in changed.lines() {
            if line.len() < 4 {
                continue;
            }
            let path = &line[3..];
            match prefix {
                Some(prefix) => {
                    if path.starts_with(prefix) {
                        return true;
                    }
                }
                None => {
                    if !path.contains('/') {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn explain_test(
        domain: &str,
        contract: &contracts::Contract,
        test: &contracts::TestCase,
        format: ContractsFormatArg,
    ) -> Result<String, String> {
        let effects = match test.kind {
            contracts::TestKind::Pure => Vec::<&str>::new(),
            contracts::TestKind::Subprocess => vec!["subprocess"],
            contracts::TestKind::Network => vec!["network"],
        };
        let payload = serde_json::json!({
            "schema_version": 1,
            "domain": domain,
            "contract_id": contract.id.0,
            "contract_title": contract.title,
            "test_id": test.id.0,
            "test_title": test.title,
            "kind": format!("{:?}", test.kind).to_ascii_lowercase(),
            "inputs_read": ["repository workspace"],
            "outputs_written": ["artifacts root when configured"],
            "effects_required": effects,
        });
        match format {
            ContractsFormatArg::Json => serde_json::to_string_pretty(&payload)
                .map_err(|e| format!("encode test explanation failed: {e}")),
            ContractsFormatArg::Human
            | ContractsFormatArg::Table
            | ContractsFormatArg::Junit
            | ContractsFormatArg::Github => Ok(format!(
                "{} {}\n{} {}\nInputs read:\n- repository workspace\nOutputs written:\n- artifacts root when configured\nEffects required:\n{}",
                contract.id.0,
                contract.title,
                test.id.0,
                test.title,
                if effects.is_empty() {
                    "- none".to_string()
                } else {
                    effects
                        .into_iter()
                        .map(|effect| format!("- {effect}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            )),
        }
    }

    fn run_one(
        descriptor: &DomainDescriptor,
        repo_root: &Path,
        common: &ContractsCommonArgs,
        contract_filter: Option<String>,
    ) -> Result<contracts::RunReport, String> {
        let run_id = std::env::var("RUN_ID")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "local".to_string());
        let mode = match common.mode {
            ContractsModeArg::Static => contracts::Mode::Static,
            ContractsModeArg::Effect => contracts::Mode::Effect,
        };
        let profile = match common.profile {
            crate::cli::ContractsProfileArg::Local => "local",
            crate::cli::ContractsProfileArg::Ci => "ci",
        };
        let artifacts_root = common.artifacts_root.clone().unwrap_or_else(|| {
            repo_root.join("artifacts")
                .join("contracts")
                .join(descriptor.name)
                .join(profile)
                .join(mode.to_string())
                .join(&run_id)
        });
        let previous_profile = std::env::var_os("BIJUX_CONTRACTS_PROFILE");
        std::env::set_var("BIJUX_CONTRACTS_PROFILE", profile);
        let options = contracts::RunOptions {
            mode,
            allow_subprocess: common.allow_subprocess,
            allow_network: common.allow_network,
            allow_k8s: common.allow_k8s,
            allow_fs_write: common.allow_fs_write,
            allow_docker_daemon: common.allow_docker_daemon,
            skip_missing_tools: common.skip_missing_tools,
            timeout_seconds: common.timeout_seconds,
            fail_fast: common.fail_fast,
            contract_filter,
            test_filter: common.filter_test.clone(),
            only_contracts: common.only_contracts.clone(),
            only_tests: common.only_tests.clone(),
            skip_contracts: common.skip_contracts.clone(),
            tags: common.tags.clone(),
            list_only: false,
            artifacts_root: Some(artifacts_root),
        };
        let result = contracts::run(descriptor.name, descriptor.contracts_fn, repo_root, &options);
        if let Some(value) = previous_profile {
            std::env::set_var("BIJUX_CONTRACTS_PROFILE", value);
        } else {
            std::env::remove_var("BIJUX_CONTRACTS_PROFILE");
        }
        result
    }

    let run = (|| -> Result<(String, i32), String> {
        if let ContractsCommand::Snapshot(args) = &command {
            let repo_root = resolve_repo_root(args.repo_root.clone())?;
            let domains = all_domains(&repo_root)?;
            let (domain_name, rows, default_rel) = match args.domain {
                ContractsSnapshotDomainArg::All => (
                    "all",
                    domains
                        .iter()
                        .flat_map(|(descriptor, registry)| {
                            contracts::registry_snapshot(descriptor.name, registry)
                        })
                        .collect::<Vec<_>>(),
                    PathBuf::from("artifacts/contracts/all/registry-snapshot.json"),
                ),
                ContractsSnapshotDomainArg::Root => (
                    "root",
                    contracts::registry_snapshot("root", domain_registry(&domains, "root")?),
                    PathBuf::from("artifacts/contracts/root/registry-snapshot.json"),
                ),
                ContractsSnapshotDomainArg::Configs => (
                    "configs",
                    contracts::registry_snapshot("configs", domain_registry(&domains, "configs")?),
                    PathBuf::from("artifacts/contracts/configs/registry-snapshot.json"),
                ),
                ContractsSnapshotDomainArg::Docs => (
                    "docs",
                    contracts::registry_snapshot("docs", domain_registry(&domains, "docs")?),
                    PathBuf::from("artifacts/contracts/docs/registry-snapshot.json"),
                ),
                ContractsSnapshotDomainArg::Docker => (
                    "docker",
                    contracts::registry_snapshot("docker", domain_registry(&domains, "docker")?),
                    PathBuf::from("artifacts/contracts/docker/registry-snapshot.json"),
                ),
                ContractsSnapshotDomainArg::Make => (
                    "make",
                    contracts::registry_snapshot("make", domain_registry(&domains, "make")?),
                    PathBuf::from("artifacts/contracts/make/registry-snapshot.json"),
                ),
                ContractsSnapshotDomainArg::Ops => (
                    "ops",
                    contracts::registry_snapshot("ops", domain_registry(&domains, "ops")?),
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

        let (repo_root, mut common, domain_names, contract_filter_override) = match &command {
            ContractsCommand::All(args) => (
                resolve_repo_root(args.repo_root.clone())?,
                args.clone(),
                vec!["root", "docker", "make", "ops", "configs", "docs"],
                None,
            ),
            ContractsCommand::Root(args) => (
                resolve_repo_root(args.repo_root.clone())?,
                args.clone(),
                vec!["root"],
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
            ContractsCommand::Snapshot(_) => unreachable!("handled above"),
        };
        apply_lane_policy(&mut common)?;

        let format = common_format(&common);
        let lints = registry_lints(&repo_root)?;
        if !lints.is_empty() {
            return Ok((render_registry_lints(&lints, format)?, 2));
        }
        require_skip_policy(&common.skip_contracts)?;

        let selected_domains = all_domains(&repo_root)?
            .into_iter()
            .filter(|(descriptor, _)| domain_names.iter().any(|name| descriptor.name == *name))
            .filter(|(descriptor, _)| {
                common.groups.is_empty()
                    || common
                        .groups
                        .iter()
                        .any(|name| descriptor.name.eq_ignore_ascii_case(name))
            })
            .filter(|(descriptor, _)| !common.changed_only || domain_has_changes(&repo_root, descriptor.name))
            .collect::<Vec<_>>();
        let catalogs = selected_domains
            .iter()
            .map(|(descriptor, registry)| (descriptor.name, registry.as_slice()))
            .collect::<Vec<_>>();
        let derived_lints = contracts::lint_contracts(&catalogs);
        if !derived_lints.is_empty() {
            return Ok((render_registry_lints(&derived_lints, format)?, 2));
        }

        if common.list || common.list_tests {
            return Ok((render_list(&selected_domains, common.list_tests, format)?, 0));
        }

        if let Some(test_id) = &common.explain_test {
            for (descriptor, registry) in &selected_domains {
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
            for (descriptor, registry) in &selected_domains {
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
            for (_, registry) in &selected_domains {
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
        for (descriptor, _) in &selected_domains {
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

        let rendered = match format {
            ContractsFormatArg::Human => {
                if reports.len() == 1 {
                    contracts::to_pretty(&reports[0])
                } else {
                    contracts::to_pretty_all(&reports)
                }
            }
            ContractsFormatArg::Table => {
                if reports.len() == 1 {
                    contracts::to_table(&reports[0])
                } else {
                    contracts::to_table_all(&reports)
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
            ContractsFormatArg::Github => contracts::to_github(&reports),
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
