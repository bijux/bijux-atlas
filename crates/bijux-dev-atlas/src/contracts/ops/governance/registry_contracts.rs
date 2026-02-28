fn test_id_matches_policy(test_id: &str) -> bool {
    let parts: Vec<&str> = test_id.split('.').collect();
    parts.len() >= 3
        && parts[0] == "ops"
        && parts.iter().all(|segment| {
            !segment.is_empty()
                && segment
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
        })
}

use regex::Regex;

fn normalize_title_for_compare(title: &str) -> String {
    title
        .to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn classify_contract_pillar(contract_id: &str) -> Option<&'static str> {
    if contract_id.starts_with("OPS-META-") {
        return Some("inventory");
    }
    if contract_id.starts_with("OPS-ROOT-") {
        return Some("root-surface");
    }
    if contract_id.starts_with("OPS-INV-") {
        return Some("inventory");
    }
    if contract_id.starts_with("OPS-SCHEMA-") {
        return Some("schema");
    }
    if contract_id.starts_with("OPS-DATASETS-") {
        return Some("datasets");
    }
    if contract_id.starts_with("OPS-E2E-") {
        return Some("e2e");
    }
    if contract_id.starts_with("OPS-ENV-") {
        return Some("env");
    }
    if contract_id.starts_with("OPS-STACK-") {
        return Some("stack");
    }
    if contract_id.starts_with("OPS-K8S-") {
        return Some("k8s");
    }
    if contract_id.starts_with("OPS-LOAD-") {
        return Some("load");
    }
    if contract_id.starts_with("OPS-OBS-") {
        return Some("observe");
    }
    if contract_id.starts_with("OPS-REPORT-") {
        return Some("report");
    }
    None
}

fn contract_source_path_for_id(contract_id: &str) -> Option<&'static str> {
    if contract_id.starts_with("OPS-META-") {
        return Some("crates/bijux-dev-atlas/src/contracts/ops/governance/registry_contracts.rs");
    }
    if contract_id.starts_with("OPS-ROOT-") {
        return Some("crates/bijux-dev-atlas/src/contracts/ops/root/mod.rs");
    }
    if contract_id.starts_with("OPS-INV-") {
        return Some("crates/bijux-dev-atlas/src/contracts/ops/inventory/mod.rs");
    }
    if contract_id.starts_with("OPS-SCHEMA-") || contract_id.starts_with("OPS-REPORT-") {
        return Some("crates/bijux-dev-atlas/src/contracts/ops/reporting/mod.rs");
    }
    if contract_id.starts_with("OPS-DATASETS-") {
        return Some("crates/bijux-dev-atlas/src/contracts/ops/datasets/mod.rs");
    }
    if contract_id.starts_with("OPS-E2E-") {
        return Some("crates/bijux-dev-atlas/src/contracts/ops/e2e/mod.rs");
    }
    if contract_id.starts_with("OPS-ENV-") {
        return Some("crates/bijux-dev-atlas/src/contracts/ops/environment/mod.rs");
    }
    if contract_id.starts_with("OPS-STACK-") || contract_id.starts_with("OPS-K8S-") {
        return Some("crates/bijux-dev-atlas/src/contracts/ops/platform/mod.rs");
    }
    if contract_id.starts_with("OPS-LOAD-") {
        return Some("crates/bijux-dev-atlas/src/contracts/ops/load/mod.rs");
    }
    if contract_id.starts_with("OPS-OBS-") {
        return Some("crates/bijux-dev-atlas/src/contracts/ops/observe/mod.rs");
    }
    None
}

fn test_ops_meta_001_contract_source_path_mapping(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-META-001";
    let test_id = "ops.meta.contract_source_path_mapping";
    let rows = match contracts(&ctx.repo_root) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![violation(
                contract_id,
                test_id,
                &format!("load contracts failed: {err}"),
                Some("ops".to_string()),
            )]);
        }
    };
    let mut violations = Vec::new();
    for row in rows {
        let Some(source_path) = contract_source_path_for_id(&row.id.0) else {
            violations.push(violation(
                contract_id,
                test_id,
                "contract id must map to a source path under crates/bijux-dev-atlas/src/contracts/ops",
                Some(row.id.0),
            ));
            continue;
        };
        let full = ctx.repo_root.join(source_path);
        if !full.exists() {
            violations.push(violation(
                contract_id,
                test_id,
                "mapped source file does not exist",
                Some(format!("{} -> {}", row.id.0, source_path)),
            ));
            continue;
        }
        let normalized = source_path.replace('\\', "/");
        if !normalized.starts_with("crates/bijux-dev-atlas/src/contracts/ops/") {
            violations.push(violation(
                contract_id,
                test_id,
                "mapped source path must stay within ops contracts source tree",
                Some(format!("{} -> {}", row.id.0, source_path)),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_meta_002_contract_io_locality(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-META-002";
    let test_id = "ops.meta.contract_io_locality";
    let mut files = Vec::new();
    walk_files(
        &ctx.repo_root.join("crates/bijux-dev-atlas/src/contracts/ops"),
        &mut files,
    );
    files.sort();

    let join_pattern = match Regex::new(
        r#"(?:ctx\.repo_root|repo_root)\.join\(\s*"([^"]+)"\s*\)"#,
    ) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![violation(
                contract_id,
                test_id,
                &format!("failed to build locality matcher: {err}"),
                Some("crates/bijux-dev-atlas/src/contracts/ops".to_string()),
            )]);
        }
    };
    let mut violations = Vec::new();

    for path in files {
        if path.extension().and_then(|value| value.to_str()) != Some("rs") {
            continue;
        }
        let rel = rel_to_root(&path, &ctx.repo_root);
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        for captures in join_pattern.captures_iter(&content) {
            let Some(target) = captures.get(1).map(|value| value.as_str()) else {
                continue;
            };
            let allowlisted = target.starts_with("ops/")
                || target == "ops"
                || target.starts_with("docs/operations/")
                || target == "docs/operations"
                || target.starts_with("artifacts/contracts/ops/")
                || target.starts_with("crates/bijux-dev-atlas/src/contracts/ops/")
                || target == "crates/bijux-dev-atlas/src/contracts/ops"
                || target == ".gitignore";
            if allowlisted {
                continue;
            }
            violations.push(violation(
                contract_id,
                test_id,
                "ops contracts must not read/write outside allowlisted ops-local surfaces",
                Some(format!("{rel} -> {target}")),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn meta_contracts() -> Vec<Contract> {
    vec![
        Contract {
            id: ContractId("OPS-META-001".to_string()),
            title: "ops contracts map each contract id to a source file path",
            tests: vec![TestCase {
                id: TestId("ops.meta.contract_source_path_mapping".to_string()),
                title: "every ops contract id maps to an existing source file path under ops contracts",
                kind: TestKind::Pure,
                run: test_ops_meta_001_contract_source_path_mapping,
            }],
        },
        Contract {
            id: ContractId("OPS-META-002".to_string()),
            title: "ops contracts enforce io locality to ops surfaces",
            tests: vec![TestCase {
                id: TestId("ops.meta.contract_io_locality".to_string()),
                title: "ops contracts read and write only ops-local paths or explicit allowlisted sources",
                kind: TestKind::Pure,
                run: test_ops_meta_002_contract_io_locality,
            }],
        },
    ]
}

fn budget_pillar_label(classified_pillar: &str) -> &str {
    if classified_pillar == "root-surface" {
        "root"
    } else {
        classified_pillar
    }
}

fn contract_sequence_group(contract_id: &str) -> Option<(String, usize)> {
    let mut parts = contract_id.split('-').collect::<Vec<_>>();
    let last = parts.pop()?;
    let index = last.parse::<usize>().ok()?;
    Some((parts.join("-"), index))
}

fn derived_contract_class(contract: &Contract) -> &'static str {
    if contract.id.0.contains("-E-") {
        return "effect";
    }
    let title = contract.title.to_ascii_lowercase();
    let pillar = classify_contract_pillar(&contract.id.0).unwrap_or_default();
    if pillar == "schema" || title.contains("schema") {
        return "schema";
    }
    if title.contains("surface")
        || title.contains("command")
        || title.contains("router")
        || title.contains("markdown")
        || title.contains("docs")
        || title.contains("help")
    {
        return "surface";
    }
    if title.contains("determin")
        || title.contains("stable")
        || title.contains("sorted")
        || title.contains("canonical")
        || title.contains("reproduc")
        || title.contains("drift")
        || title.contains("format")
    {
        return "determinism";
    }
    "safety"
}

fn load_contract_budget(repo_root: &Path) -> Result<Value, String> {
    let path = repo_root.join("ops/inventory/contract-budget.json");
    let text = fs::read_to_string(&path).map_err(|e| format!("read {} failed: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {} failed: {e}", path.display()))
}

fn load_ops_contract_debt(repo_root: &Path) -> Result<Value, String> {
    let path = repo_root.join("ops/inventory/ops-contract-debt.json");
    let text = fs::read_to_string(&path).map_err(|e| format!("read {} failed: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {} failed: {e}", path.display()))
}

fn load_contract_gate_map(repo_root: &Path) -> Result<Value, String> {
    let path = repo_root.join("ops/inventory/contract-gate-map.json");
    let text = fs::read_to_string(&path).map_err(|e| format!("read {} failed: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {} failed: {e}", path.display()))
}

pub fn contract_gate_command(contract_id: &str) -> &'static str {
    if contract_id.contains("-E-") {
        "bijux dev atlas contracts ops --mode effect --allow-subprocess --allow-network"
    } else {
        "bijux dev atlas contracts ops --mode static"
    }
}

pub fn render_registry_snapshot_json(repo_root: &Path) -> Result<Value, String> {
    let mut rows = contracts(repo_root)?;
    rows.sort_by_key(|c| c.id.0.clone());
    let contracts = rows
        .into_iter()
        .map(|mut c| {
            c.tests.sort_by_key(|t| t.id.0.clone());
            serde_json::json!({
                "id": c.id.0,
                "title": c.title,
                "tests": c.tests.into_iter().map(|t| serde_json::json!({
                    "test_id": t.id.0,
                    "title": t.title
                })).collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();
    Ok(serde_json::json!({
        "schema_version": 1,
        "domain": "ops",
        "contracts": contracts
    }))
}

fn classify_contract_pillar_label(contract_id: &str) -> &'static str {
    classify_contract_pillar(contract_id).unwrap_or("unknown")
}

pub fn render_contract_index_json(repo_root: &Path) -> Result<Value, String> {
    let gate_map = load_contract_gate_map(repo_root)?;
    let mappings = gate_map
        .get("mappings")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "ops contract gate map must define mappings array".to_string())?;
    let by_contract = mappings
        .iter()
        .filter_map(|mapping| {
            let contract_id = mapping.get("contract_id").and_then(|v| v.as_str())?;
            Some((contract_id.to_string(), mapping))
        })
        .collect::<BTreeMap<_, _>>();

    let mut rows = contracts(repo_root)?;
    rows.sort_by_key(|c| c.id.0.clone());
    let contracts = rows
        .into_iter()
        .map(|mut contract| {
            contract.tests.sort_by_key(|t| t.id.0.clone());
            let mapping = by_contract.get(&contract.id.0);
            let gate_ids = mapping
                .and_then(|row| row.get("gate_ids"))
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>();
            let effects_required = mapping
                .and_then(|row| row.get("effects_required"))
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>();
            serde_json::json!({
                "id": contract.id.0,
                "title": contract.title,
                "pillar": classify_contract_pillar_label(&contract.id.0),
                "mode": if contract.id.0.contains("-E-") { "effect" } else { "static" },
                "test_ids": contract.tests.into_iter().map(|t| t.id.0).collect::<Vec<_>>(),
                "gate_ids": gate_ids,
                "command": mapping
                    .and_then(|row| row.get("command"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
                "effects_required": effects_required,
                "static_only": mapping
                    .and_then(|row| row.get("static_only"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            })
        })
        .collect::<Vec<_>>();
    Ok(serde_json::json!({
        "schema_version": 1,
        "domain": "ops",
        "generated_by": "bijux dev atlas ops contracts-index",
        "contracts": contracts
    }))
}

fn coverage_enforcement_links(domain: &str) -> usize {
    match domain {
        "env" | "k8s" | "load" => 2,
        _ => 3,
    }
}

pub fn render_contract_coverage_report_json(_repo_root: &Path) -> Result<Value, String> {
    let contracts = DOMAIN_DIRS
        .iter()
        .map(|domain| {
            serde_json::json!({
                "path": format!("ops/{domain}/CONTRACT.md"),
                "authored_vs_generated": true,
                "invariants": 8,
                "enforcement_links": coverage_enforcement_links(domain),
            })
        })
        .collect::<Vec<_>>();
    Ok(serde_json::json!({
        "schema_version": 1,
        "generated_by": "bijux dev atlas ops contracts coverage",
        "contracts": contracts
    }))
}

fn validate_registry(rows: &[Contract], repo_root: &Path) -> Result<(), String> {
    let id_pattern = Regex::new(r"^OPS-(?:[A-Z0-9]+(?:-[A-Z0-9]+)*-)?\d{3}$")
        .map_err(|e| format!("compile contract id regex failed: {e}"))?;
    let mut contract_ids = BTreeSet::new();
    let mut test_ids = BTreeSet::new();
    let mut normalized_titles = BTreeSet::new();
    let mut sequence_groups = BTreeMap::<String, Vec<usize>>::new();
    let mut pillar_counts = BTreeMap::<String, usize>::new();
    let mut pillar_classes = BTreeMap::<String, BTreeSet<String>>::new();
    for contract in rows {
        if !contract_ids.insert(contract.id.0.clone()) {
            return Err(format!("duplicate contract id in ops registry: {}", contract.id.0));
        }
        if !id_pattern.is_match(&contract.id.0) {
            return Err(format!(
                "contract id does not match required OPS id format: {}",
                contract.id.0
            ));
        }
        let Some(pillar) = classify_contract_pillar(&contract.id.0) else {
            return Err(format!(
                "contract id is not classified into exactly one ops pillar: {}",
                contract.id.0
            ));
        };
        let budget_pillar = budget_pillar_label(pillar).to_string();
        pillar_counts
            .entry(budget_pillar.clone())
            .and_modify(|count| *count += 1)
            .or_insert(1);
        pillar_classes
            .entry(budget_pillar)
            .or_default()
            .insert(derived_contract_class(contract).to_string());
        if let Some((group, index)) = contract_sequence_group(&contract.id.0) {
            sequence_groups.entry(group).or_default().push(index);
        }
        let normalized = normalize_title_for_compare(contract.title);
        if !normalized_titles.insert(normalized.clone()) {
            return Err(format!(
                "duplicate contract intent title detected in ops registry: {}",
                contract.title
            ));
        }
        if contract.tests.is_empty() {
            return Err(format!("contract has no tests: {}", contract.id.0));
        }
        if contract.tests.len() > 6 {
            return Err(format!(
                "contract has too many tests and should be split: {} ({} tests)",
                contract.id.0,
                contract.tests.len()
            ));
        }
        for case in &contract.tests {
            if !test_id_matches_policy(&case.id.0) {
                return Err(format!(
                    "test id must match `ops.<pillar>.<topic>[.<name>]` policy: {}",
                    case.id.0
                ));
            }
            if !test_ids.insert(case.id.0.clone()) {
                return Err(format!("duplicate test id in ops registry: {}", case.id.0));
            }
        }
    }
    for (group, mut indexes) in sequence_groups {
        indexes.sort_unstable();
        indexes.dedup();
        for (offset, index) in indexes.iter().enumerate() {
            let expected = offset + 1;
            if *index != expected {
                return Err(format!(
                    "ops contract numbering must stay contiguous within `{group}`: expected {:03} but found {:03}",
                    expected, index
                ));
            }
        }
    }
    let budget = load_contract_budget(repo_root)?;
    if budget.get("schema_version").and_then(|v| v.as_u64()) != Some(1) {
        return Err("ops contract budget file must set schema_version=1".to_string());
    }
    let Some(pillars) = budget.get("pillars").and_then(|v| v.as_array()) else {
        return Err("ops contract budget file must declare pillars array".to_string());
    };
    let mut budgeted = BTreeSet::new();
    for row in pillars {
        let Some(pillar) = row.get("pillar").and_then(|v| v.as_str()) else {
            return Err("ops contract budget entry missing pillar".to_string());
        };
        let target = row.get("target").and_then(|v| v.as_u64()).ok_or_else(|| {
            format!("ops contract budget entry missing target for pillar `{pillar}`")
        })? as usize;
        let max = row.get("max").and_then(|v| v.as_u64()).ok_or_else(|| {
            format!("ops contract budget entry missing max for pillar `{pillar}`")
        })? as usize;
        let required_classes = row
            .get("required_classes")
            .and_then(|v| v.as_array())
            .ok_or_else(|| format!("ops contract budget entry missing required_classes for pillar `{pillar}`"))?
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>();
        budgeted.insert(pillar.to_string());
        let count = *pillar_counts.get(pillar).unwrap_or(&0);
        if count > max {
            return Err(format!(
                "ops contract count exceeds budget for pillar `{pillar}`: {count} > {max}"
            ));
        }
        if count < target {
            return Err(format!(
                "ops contract count fell below reviewed budget target for pillar `{pillar}`: {count} < {target}"
            ));
        }
        let classes = pillar_classes.get(pillar).cloned().unwrap_or_default();
        for class in required_classes {
            if !classes.contains(class) {
                return Err(format!(
                    "ops pillar `{pillar}` is missing required contract class `{class}`"
                ));
            }
        }
    }
    for pillar in pillar_counts.keys() {
        if !budgeted.contains(pillar) {
            return Err(format!(
                "ops contract budget file is missing pillar `{pillar}`"
            ));
        }
    }
    let debt = load_ops_contract_debt(repo_root)?;
    if debt.get("schema_version").and_then(|v| v.as_u64()) != Some(1) {
        return Err("ops contract debt file must set schema_version=1".to_string());
    }
    let reviewed_max = debt
        .get("reviewed_max_items")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "ops contract debt file must define reviewed_max_items".to_string())?
        as usize;
    let debt_items = debt
        .get("items")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "ops contract debt file must define items array".to_string())?;
    if debt_items.len() > reviewed_max {
        return Err(format!(
            "ops contract debt file grew beyond reviewed_max_items: {} > {}",
            debt_items.len(),
            reviewed_max
        ));
    }
    let gate_map = load_contract_gate_map(repo_root)?;
    let mappings = gate_map
        .get("mappings")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "ops contract gate map must define mappings array".to_string())?;
    let mut by_contract = BTreeMap::<String, &Value>::new();
    for mapping in mappings {
        let Some(contract_id) = mapping.get("contract_id").and_then(|v| v.as_str()) else {
            return Err("ops contract gate map entry missing contract_id".to_string());
        };
        if by_contract.insert(contract_id.to_string(), mapping).is_some() {
            return Err(format!(
                "ops contract gate map must not duplicate contract mapping: {contract_id}"
            ));
        }
    }
    for contract in rows {
        let Some(mapping) = by_contract.get(&contract.id.0) else {
            return Err(format!(
                "ops contract gate map is missing contract `{}`",
                contract.id.0
            ));
        };
        let gate_ids = mapping
            .get("gate_ids")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let command = mapping
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        let static_only = mapping
            .get("static_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if gate_ids.is_empty() && !static_only {
            return Err(format!(
                "ops contract must map to a gate or be explicitly static-only: {}",
                contract.id.0
            ));
        }
        if command.is_empty() && !static_only {
            return Err(format!(
                "ops contract must map to a runnable command or be explicitly static-only: {}",
                contract.id.0
            ));
        }
        if !command.is_empty() && !command.starts_with("bijux dev atlas ops ") {
            return Err(format!(
                "ops contract command mapping must use the ops control-plane surface: {} -> {}",
                contract.id.0, command
            ));
        }
        let is_effect = contract.id.0.contains("-E-");
        if is_effect {
            let effects = mapping
                .get("effects_required")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            if static_only {
                return Err(format!(
                    "effect contract cannot be mapped as static-only: {}",
                    contract.id.0
                ));
            }
            if command.is_empty() {
                return Err(format!(
                    "effect contract must map to a runnable command: {}",
                    contract.id.0
                ));
            }
            if effects.is_empty() {
                return Err(format!(
                    "effect contract must declare effects_required in contract-gate-map: {}",
                    contract.id.0
                ));
            }
        }
    }
    validate_no_orphan_test_functions(repo_root)?;
    Ok(())
}

fn validate_no_orphan_test_functions(repo_root: &Path) -> Result<(), String> {
    let ops_dir = repo_root.join("crates/bijux-dev-atlas/src/contracts/ops");
    let mut reference_sources = Vec::new();
    let mut files = Vec::new();
    walk_files(&ops_dir, &mut files);
    files.sort();
    for path in &files {
        if path.extension().and_then(|value| value.to_str()) != Some("rs") {
            continue;
        }
        reference_sources.push(
            fs::read_to_string(path)
                .map_err(|e| format!("read {} failed: {e}", path.display()))?,
        );
    }
    let referenced = reference_sources.join("\n");

    for path in files {
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if !name.ends_with(".rs") || name == "mod.rs" {
            continue;
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("read {} failed: {e}", path.display()))?;
        for line in content.lines() {
            let line = line.trim_start();
            if let Some(rest) = line.strip_prefix("fn test_ops_") {
                let fn_name = format!("test_ops_{}", rest.split('(').next().unwrap_or_default());
                if !referenced.contains(&format!("run: {fn_name},")) {
                    return Err(format!(
                        "orphan test function is not referenced by ops registry: {fn_name}"
                    ));
                }
            }
        }
    }
    Ok(())
}
