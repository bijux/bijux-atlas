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

fn validate_registry(rows: &[Contract], repo_root: &Path) -> Result<(), String> {
    let id_pattern = Regex::new(r"^OPS-(?:[A-Z0-9]+(?:-[A-Z0-9]+)*-)?\d{3}$")
        .map_err(|e| format!("compile contract id regex failed: {e}"))?;
    let mut contract_ids = BTreeSet::new();
    let mut test_ids = BTreeSet::new();
    let mut normalized_titles = BTreeSet::new();
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
        if classify_contract_pillar(&contract.id.0).is_none() {
            return Err(format!(
                "contract id is not classified into exactly one ops pillar: {}",
                contract.id.0
            ));
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
    validate_no_orphan_test_functions(repo_root)?;
    Ok(())
}

fn validate_no_orphan_test_functions(repo_root: &Path) -> Result<(), String> {
    let ops_dir = repo_root.join("crates/bijux-dev-atlas/src/contracts/ops");
    let mut reference_sources = Vec::new();
    reference_sources.push(
        fs::read_to_string(repo_root.join("crates/bijux-dev-atlas/src/contracts/ops/mod.rs"))
            .map_err(|e| format!("read contracts ops module failed: {e}"))?,
    );
    let mut assembly_files = sorted_dir_entries(&ops_dir);
    assembly_files.retain(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                matches!(
                    name,
                    "ops_registry.inc.rs"
                        | "root_surface.inc.rs"
                        | "inventory.inc.rs"
                        | "schema.inc.rs"
                        | "datasets.inc.rs"
                        | "e2e.inc.rs"
                        | "env.inc.rs"
                        | "stack.inc.rs"
                        | "k8s.inc.rs"
                        | "observe.inc.rs"
                        | "load.inc.rs"
                        | "report.inc.rs"
                        | "pillars.inc.rs"
                )
            })
    });
    for path in assembly_files {
        reference_sources.push(
            fs::read_to_string(&path)
                .map_err(|e| format!("read {} failed: {e}", path.display()))?,
        );
    }
    let referenced = reference_sources.join("\n");

    let mut files = Vec::new();
    walk_files(&ops_dir, &mut files);
    files.sort();
    for path in files {
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if !name.ends_with(".inc.rs") || name == "ops_registry.inc.rs" {
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
