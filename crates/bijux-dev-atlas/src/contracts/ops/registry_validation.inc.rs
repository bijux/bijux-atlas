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
    let mut contract_ids = BTreeSet::new();
    let mut test_ids = BTreeSet::new();
    for contract in rows {
        if !contract_ids.insert(contract.id.0.clone()) {
            return Err(format!("duplicate contract id in ops registry: {}", contract.id.0));
        }
        if contract.tests.is_empty() {
            return Err(format!("contract has no tests: {}", contract.id.0));
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
    let registry = fs::read_to_string(
        repo_root.join("crates/bijux-dev-atlas/src/contracts/ops/ops_registry.inc.rs"),
    )
    .map_err(|e| format!("read ops registry source failed: {e}"))?;

    let mut files = Vec::new();
    walk_files(&repo_root.join("crates/bijux-dev-atlas/src/contracts/ops"), &mut files);
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
                if !registry.contains(&format!("run: {fn_name},")) {
                    return Err(format!(
                        "orphan test function is not referenced by ops registry: {fn_name}"
                    ));
                }
            }
        }
    }
    Ok(())
}
