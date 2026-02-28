fn ops_surface_command_set(root: &Path) -> BTreeSet<String> {
    read_json(&root.join("ops/inventory/surfaces.json"))
        .and_then(|v| v.get("bijux-dev-atlas_commands").cloned())
        .and_then(|v| v.as_array().cloned())
        .map(|rows| {
            rows.into_iter()
                .filter_map(|v| v.as_str().map(ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn ops_surface_actions_to_command_set(root: &Path) -> BTreeSet<String> {
    read_json(&root.join("ops/inventory/surfaces.json"))
        .and_then(|v| v.get("actions").cloned())
        .and_then(|v| v.as_array().cloned())
        .map(|rows| {
            rows.into_iter()
                .filter_map(|row| row.get("command").and_then(|v| v.as_array()).cloned())
                .map(|parts| {
                    parts
                        .into_iter()
                        .filter_map(|v| v.as_str().map(ToOwned::to_owned))
                        .collect::<Vec<_>>()
                        .join(" ")
                        .replace("bijux-dev-atlas", "bijux dev atlas")
                })
                .collect()
        })
        .unwrap_or_default()
}

fn test_ops_root_surface_001_required_commands_exist(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-SURFACE-001";
    let test_id = "ops.root_surface.required_commands_exist";
    let known = ops_surface_command_set(&ctx.repo_root);
    let required = [
        "bijux dev atlas ops stack up",
        "bijux dev atlas ops stack down",
        "bijux dev atlas ops k8s render",
        "bijux dev atlas ops k8s check",
        "bijux dev atlas ops load run",
        "bijux dev atlas ops observe verify",
        "bijux dev atlas ops list",
    ];
    let mut violations = Vec::new();
    for command in required {
        if !known.contains(command) {
            violations.push(violation(
                contract_id,
                test_id,
                "required ops command is missing from command surface",
                Some(command.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_root_surface_002_no_hidden_commands(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-SURFACE-002";
    let test_id = "ops.root_surface.no_hidden_commands";
    let listed = ops_surface_command_set(&ctx.repo_root);
    let from_actions = ops_surface_actions_to_command_set(&ctx.repo_root);
    let mut violations = Vec::new();
    for command in listed.difference(&from_actions) {
        violations.push(violation(
            contract_id,
            test_id,
            "command is listed but has no matching action dispatch entry",
            Some(command.to_string()),
        ));
    }
    for command in from_actions.difference(&listed) {
        violations.push(violation(
            contract_id,
            test_id,
            "action dispatch command is missing from listed command surface",
            Some(command.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_root_surface_003_surface_ordering_deterministic(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-SURFACE-003";
    let test_id = "ops.root_surface.surface_ordering_deterministic";
    let Some(surface_json) = read_json(&ctx.repo_root.join("ops/inventory/surfaces.json")) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "surfaces inventory must exist and be valid json",
            Some("ops/inventory/surfaces.json".to_string()),
        )]);
    };
    let Some(commands) = surface_json
        .get("bijux-dev-atlas_commands")
        .and_then(|v| v.as_array())
    else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "surfaces inventory must define bijux-dev-atlas_commands",
            Some("ops/inventory/surfaces.json".to_string()),
        )]);
    };
    let values: Vec<String> = commands
        .iter()
        .filter_map(|v| v.as_str().map(ToOwned::to_owned))
        .collect();
    let mut sorted = values.clone();
    sorted.sort();
    if values == sorted {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "bijux-dev-atlas_commands must be sorted deterministically",
            Some("ops/inventory/surfaces.json".to_string()),
        )])
    }
}

fn test_ops_root_surface_004_commands_declare_effects(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-SURFACE-004";
    let test_id = "ops.root_surface.commands_declare_effects";
    let map = match read_contract_gate_map(&ctx.repo_root) {
        Ok(v) => v,
        Err(_) => {
            return TestResult::Fail(vec![violation(
                contract_id,
                test_id,
                "contract-gate-map must exist and be valid json",
                Some("ops/inventory/contract-gate-map.json".to_string()),
            )]);
        }
    };
    let mut has_seen = BTreeSet::new();
    let mut violations = Vec::new();
    for item in map
        .get("mappings")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let command = item
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if command.is_empty() {
            continue;
        }
        has_seen.insert(command.to_string());
        if item.get("effects_required").and_then(|v| v.as_array()).is_none() {
            violations.push(violation(
                contract_id,
                test_id,
                "mapped command must declare effects_required array",
                Some(command.to_string()),
            ));
        }
    }
    if has_seen.is_empty() {
        violations.push(violation(
            contract_id,
            test_id,
            "contract-gate-map must declare command-level effect requirements",
            Some("ops/inventory/contract-gate-map.json".to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_root_surface_005_commands_grouped_by_pillar(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-SURFACE-005";
    let test_id = "ops.root_surface.commands_grouped_by_pillar";
    let Some(surface_json) = read_json(&ctx.repo_root.join("ops/inventory/surfaces.json")) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "surfaces inventory must exist and be valid json",
            Some("ops/inventory/surfaces.json".to_string()),
        )]);
    };
    let allowlist: BTreeSet<&str> = BTreeSet::from([
        "actions",
        "cache",
        "datasets",
        "deploy",
        "e2e",
        "env",
        "gen",
        "k8s",
        "kind",
        "load",
        "observe",
        "pins",
        "stack",
        "root",
    ]);
    let required_domains: BTreeSet<&str> = BTreeSet::from([
        "datasets",
        "e2e",
        "env",
        "k8s",
        "load",
        "observe",
        "stack",
        "root",
    ]);
    let mut seen_domains = BTreeSet::new();
    let mut violations = Vec::new();
    for action in surface_json
        .get("actions")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let action_id = action.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        let domain = action
            .get("domain")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if domain.is_empty() || !allowlist.contains(domain) {
            violations.push(violation(
                contract_id,
                test_id,
                "command action domain must be an approved pillar group",
                Some(action_id.to_string()),
            ));
            continue;
        }
        seen_domains.insert(domain.to_string());
        let argv = action
            .get("argv")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(ToOwned::to_owned))
            .collect::<Vec<_>>();
        if domain == "root" {
            if argv.is_empty() {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "root command group must define a non-empty argv surface",
                    Some(action_id.to_string()),
                ));
            }
        } else if argv.first().map(|segment| segment.as_str()) != Some(domain) {
            violations.push(violation(
                contract_id,
                test_id,
                "command argv must start with its declared command group",
                Some(action_id.to_string()),
            ));
        }
    }
    for domain in required_domains {
        if !seen_domains.contains(domain) {
            violations.push(violation(
                contract_id,
                test_id,
                "required ops command group is missing from the registered surface",
                Some(domain.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_root_surface_006_forbid_adhoc_command_groups(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-SURFACE-006";
    let test_id = "ops.root_surface.forbid_adhoc_command_groups";
    let Some(surface_json) = read_json(&ctx.repo_root.join("ops/inventory/surfaces.json")) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "surfaces inventory must exist and be valid json",
            Some("ops/inventory/surfaces.json".to_string()),
        )]);
    };
    let denylist = ["misc", "util", "utils", "tmp", "legacy"];
    let mut violations = Vec::new();
    for action in surface_json
        .get("actions")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let action_id = action.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        let domain = action
            .get("domain")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if denylist.contains(&domain) {
            violations.push(violation(
                contract_id,
                test_id,
                "ad-hoc command groups are forbidden",
                Some(action_id.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_root_surface_007_command_purpose_defined(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-SURFACE-007";
    let test_id = "ops.root_surface.command_purpose_defined";
    let Some(surface_json) = read_json(&ctx.repo_root.join("ops/inventory/surfaces.json")) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "surfaces inventory must exist and be valid json",
            Some("ops/inventory/surfaces.json".to_string()),
        )]);
    };
    let mut violations = Vec::new();
    for action in surface_json
        .get("actions")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let action_id = action.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        let purpose = action
            .get("purpose")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .trim();
        if purpose.is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                "each command action must declare a stable purpose string",
                Some(action_id.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_root_surface_008_command_supports_json(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-SURFACE-008";
    let test_id = "ops.root_surface.command_supports_json";
    let Some(surface_json) = read_json(&ctx.repo_root.join("ops/inventory/surfaces.json")) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "surfaces inventory must exist and be valid json",
            Some("ops/inventory/surfaces.json".to_string()),
        )]);
    };
    let mut violations = Vec::new();
    for action in surface_json
        .get("actions")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let action_id = action.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        let supports_json = action
            .get("supports_json")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !supports_json {
            violations.push(violation(
                contract_id,
                test_id,
                "each command action must declare supports_json=true",
                Some(action_id.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_root_surface_009_command_dry_run_policy(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-SURFACE-009";
    let test_id = "ops.root_surface.command_dry_run_policy";
    let Some(surface_json) = read_json(&ctx.repo_root.join("ops/inventory/surfaces.json")) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "surfaces inventory must exist and be valid json",
            Some("ops/inventory/surfaces.json".to_string()),
        )]);
    };
    let mut violations = Vec::new();
    for action in surface_json
        .get("actions")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let action_id = action.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        let policy = action
            .get("dry_run")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let artifacts_policy = action
            .get("artifacts_policy")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if policy != "required" && policy != "optional" && policy != "not_applicable" {
            violations.push(violation(
                contract_id,
                test_id,
                "dry_run policy must be required|optional|not_applicable",
                Some(action_id.to_string()),
            ));
            continue;
        }
        if (policy == "required" || policy == "optional") && artifacts_policy != "artifacts_root_only" {
            violations.push(violation(
                contract_id,
                test_id,
                "commands that support dry-run must write artifacts only under artifacts_root",
                Some(action_id.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_root_surface_010_artifacts_root_policy(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-SURFACE-010";
    let test_id = "ops.root_surface.artifacts_root_policy";
    let Some(surface_json) = read_json(&ctx.repo_root.join("ops/inventory/surfaces.json")) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "surfaces inventory must exist and be valid json",
            Some("ops/inventory/surfaces.json".to_string()),
        )]);
    };
    let mut violations = Vec::new();
    for action in surface_json
        .get("actions")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let action_id = action.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        let policy = action
            .get("artifacts_policy")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let dry_run = action
            .get("dry_run")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if policy != "artifacts_root_only" && policy != "none" {
            violations.push(violation(
                contract_id,
                test_id,
                "artifacts_policy must be artifacts_root_only or none",
                Some(action_id.to_string()),
            ));
            continue;
        }
        if dry_run != "not_applicable" && policy != "artifacts_root_only" {
            violations.push(violation(
                contract_id,
                test_id,
                "commands with runtime execution must not write outside artifacts_root",
                Some(action_id.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

