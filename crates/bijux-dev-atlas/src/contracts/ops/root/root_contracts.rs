fn ops_markdown_allowed(rel: &str) -> bool {
    if rel == "ops/README.md" || rel == "ops/CONTRACT.md" {
        return true;
    }
    for domain in DOMAIN_DIRS {
        if rel == format!("ops/{domain}/README.md") || rel == format!("ops/{domain}/CONTRACT.md") {
            return true;
        }
    }
    false
}

fn inventory_paths(repo_root: &Path) -> (BTreeSet<String>, BTreeSet<String>) {
    let authoritative_file_list = repo_root.join("ops/inventory/authoritative-file-list.json");
    let contracts_map = repo_root.join("ops/inventory/contracts-map.json");

    let mut authoritative = BTreeSet::new();
    let mut contracts_map_items = BTreeSet::new();

    if let Some(value) = read_json(&authoritative_file_list) {
        if let Some(paths) = value.get("authoritative_paths").and_then(|v| v.as_array()) {
            for path in paths {
                if let Some(path) = path.as_str() {
                    authoritative.insert(path.to_string());
                }
            }
        }
    }

    if let Some(value) = read_json(&contracts_map) {
        if let Some(items) = value.get("items").and_then(|v| v.as_array()) {
            for item in items {
                if let Some(path) = item.get("path").and_then(|v| v.as_str()) {
                    contracts_map_items.insert(path.to_string());
                }
            }
        }
    }

    (authoritative, contracts_map_items)
}

#[derive(Debug, Deserialize)]
struct InventoryPillarsDoc {
    pillars: Vec<InventoryPillarRow>,
}

#[derive(Debug, Deserialize)]
struct InventoryPillarRow {
    id: String,
}

fn read_pillars_doc(repo_root: &Path) -> Result<InventoryPillarsDoc, String> {
    let path = repo_root.join("ops/inventory/pillars.json");
    let content =
        fs::read_to_string(&path).map_err(|e| format!("read {} failed: {e}", path.display()))?;
    serde_json::from_str(&content).map_err(|e| format!("parse {} failed: {e}", path.display()))
}

fn test_ops_000_allowed_root_files(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-017";
    let test_id = "ops.dir.allowed_root_files";
    let root = ops_root(&ctx.repo_root);
    let Ok(entries) = std::fs::read_dir(&root) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "ops root directory is missing",
            Some("ops".to_string()),
        )]);
    };
    let allowed_files = BTreeSet::from(["README.md", "CONTRACT.md"]);
    let mut violations = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
                continue;
            };
            if !allowed_files.contains(name) {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "unexpected root file under ops/",
                    Some(rel_to_root(&path, &ctx.repo_root)),
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_000_forbid_extra_markdown_root(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-017";
    let test_id = "ops.dir.forbid_extra_markdown_root";
    let root = ops_root(&ctx.repo_root);
    let Ok(entries) = std::fs::read_dir(&root) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "ops root directory is missing",
            Some("ops".to_string()),
        )]);
    };
    let allowed_markdown = BTreeSet::from(["README.md", "CONTRACT.md"]);
    let mut violations = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let is_markdown = path
            .extension()
            .and_then(|v| v.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"));
        if !is_markdown {
            continue;
        }
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if !allowed_markdown.contains(name) {
            violations.push(violation(
                contract_id,
                test_id,
                "only ops/README.md and ops/CONTRACT.md are allowed at ops root",
                Some(rel_to_root(&path, &ctx.repo_root)),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_000_allow_only_known_domain_dirs(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-017";
    let test_id = "ops.dir.allow_only_known_domain_dirs";
    let root = ops_root(&ctx.repo_root);
    let Ok(entries) = std::fs::read_dir(&root) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "ops root directory is missing",
            Some("ops".to_string()),
        )]);
    };
    let mut allowed = BTreeSet::new();
    for name in DOMAIN_DIRS {
        allowed.insert(*name);
    }
    allowed.insert("_generated");
    allowed.insert("_generated.example");
    allowed.insert("policy");
    let mut violations = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if !allowed.contains(name) {
            violations.push(violation(
                contract_id,
                test_id,
                "unknown directory under ops root",
                Some(rel_to_root(&path, &ctx.repo_root)),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_000_forbid_extra_markdown_recursive(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-017";
    let test_id = "ops.dir.forbid_extra_markdown_recursive";
    let ops_root = ops_root(&ctx.repo_root);
    let mut files = Vec::new();
    walk_files(&ops_root, &mut files);
    files.sort();

    let mut violations = Vec::new();
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if rel.starts_with("ops/_generated/") || rel.starts_with("ops/_generated.example/") {
            continue;
        }
        let is_markdown = path
            .extension()
            .and_then(|v| v.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"));
        if is_markdown && !ops_markdown_allowed(&rel) {
            violations.push(violation(
                contract_id,
                test_id,
                "markdown file outside allowed ops surface",
                Some(rel),
            ));
        }
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_001_generated_runtime_allowed_files(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-018";
    let test_id = "ops.generated.runtime.allowed_files";
    let root = ctx.repo_root.join("ops/_generated");
    if !root.exists() {
        return TestResult::Pass;
    }

    let mut files = Vec::new();
    walk_files(&root, &mut files);
    files.sort();

    let allowed_extensions = BTreeSet::from(["json", "gitkeep"]);
    let mut violations = Vec::new();
    for path in files {
        let name = path.file_name().and_then(|v| v.to_str()).unwrap_or("");
        if name == ".gitkeep" {
            continue;
        }
        let ext = path.extension().and_then(|v| v.to_str()).unwrap_or("");
        if !allowed_extensions.contains(ext) {
            violations.push(violation(
                contract_id,
                test_id,
                "ops/_generated allows only .json artifacts",
                Some(rel_to_root(&path, &ctx.repo_root)),
            ));
        }
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_001_generated_example_allowed_files(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-018";
    let test_id = "ops.generated.example.allowed_files";
    let root = ctx.repo_root.join("ops/_generated.example");
    if !root.exists() {
        return TestResult::Pass;
    }

    let mut files = Vec::new();
    walk_files(&root, &mut files);
    files.sort();

    let allowed_extensions = BTreeSet::from(["json", "gitkeep"]);
    let mut violations = Vec::new();
    for path in files {
        let name = path.file_name().and_then(|v| v.to_str()).unwrap_or("");
        if name == ".gitkeep" {
            continue;
        }
        let ext = path.extension().and_then(|v| v.to_str()).unwrap_or("");
        if !allowed_extensions.contains(ext) {
            violations.push(violation(
                contract_id,
                test_id,
                "ops/_generated.example allows only .json artifacts",
                Some(rel_to_root(&path, &ctx.repo_root)),
            ));
        }
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_001_generated_runtime_forbid_example_files(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-018";
    let test_id = "ops.generated.runtime.no_example_files";
    let root = ctx.repo_root.join("ops/_generated");
    if !root.exists() {
        return TestResult::Pass;
    }

    let mut files = Vec::new();
    walk_files(&root, &mut files);
    files.sort();

    let mut violations = Vec::new();
    for path in files {
        let name = path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if name.contains("example") {
            violations.push(violation(
                contract_id,
                test_id,
                "ops/_generated must not contain example artifacts",
                Some(rel_to_root(&path, &ctx.repo_root)),
            ));
        }
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_002_domain_required_files(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-019";
    let test_id = "ops.domain.required_contract_and_readme";
    let mut violations = Vec::new();

    for domain in DOMAIN_DIRS {
        let domain_root = ctx.repo_root.join("ops").join(domain);
        for required in ["README.md", "CONTRACT.md"] {
            let path = domain_root.join(required);
            if !path.exists() {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "ops domain must include README.md and CONTRACT.md",
                    Some(rel_to_root(&path, &ctx.repo_root)),
                ));
            }
        }
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_002_forbid_legacy_domain_docs(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-019";
    let test_id = "ops.domain.forbid_legacy_docs";
    let mut violations = Vec::new();

    for domain in DOMAIN_DIRS {
        let domain_root = ctx.repo_root.join("ops").join(domain);
        for forbidden in ["INDEX.md", "OWNER.md", "REQUIRED_FILES.md"] {
            let path = domain_root.join(forbidden);
            if path.exists() {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "legacy domain markdown docs are forbidden",
                    Some(rel_to_root(&path, &ctx.repo_root)),
                ));
            }
        }
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_003_readme_markdown_budget(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-020";
    let test_id = "ops.markdown_budget.readme";
    let mut violations = Vec::new();

    let root_readme = ctx.repo_root.join("ops/README.md");
    let lines = markdown_line_count(&root_readme);
    if lines > 200 {
        violations.push(violation(
            contract_id,
            test_id,
            "ops/README.md exceeds line budget (max 200)",
            Some("ops/README.md".to_string()),
        ));
    }

    for domain in DOMAIN_DIRS {
        let path = ctx.repo_root.join("ops").join(domain).join("README.md");
        let lines = markdown_line_count(&path);
        if lines > 200 {
            violations.push(violation(
                contract_id,
                test_id,
                "domain README.md exceeds line budget (max 200)",
                Some(rel_to_root(&path, &ctx.repo_root)),
            ));
        }
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_003_contract_markdown_budget(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-020";
    let test_id = "ops.markdown_budget.contract";
    let mut violations = Vec::new();

    let root_contract = ctx.repo_root.join("ops/CONTRACT.md");
    let lines = markdown_line_count(&root_contract);
    if lines > 900 {
        violations.push(violation(
            contract_id,
            test_id,
            "ops/CONTRACT.md exceeds line budget (max 900)",
            Some("ops/CONTRACT.md".to_string()),
        ));
    }

    for domain in DOMAIN_DIRS {
        let path = ctx.repo_root.join("ops").join(domain).join("CONTRACT.md");
        let lines = markdown_line_count(&path);
        if lines > 400 {
            violations.push(violation(
                contract_id,
                test_id,
                "domain CONTRACT.md exceeds line budget (max 400)",
                Some(rel_to_root(&path, &ctx.repo_root)),
            ));
        }
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_004_readme_ssot_boundary(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-021";
    let test_id = "ops.docs.readme_ssot_boundary";
    let path = ctx.repo_root.join("ops/README.md");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "ops/README.md is missing",
            Some("ops/README.md".to_string()),
        )]);
    };

    let mut violations = Vec::new();
    if !content.contains("docs/operations") {
        violations.push(violation(
            contract_id,
            test_id,
            "ops/README.md must link to docs/operations for walkthroughs",
            Some("ops/README.md".to_string()),
        ));
    }
    if !content.contains("bijux dev atlas contracts ops") {
        violations.push(violation(
            contract_id,
            test_id,
            "ops/README.md must include the contracts runner command",
            Some("ops/README.md".to_string()),
        ));
    }
    if content.contains("## Run") || content.contains("```") {
        violations.push(violation(
            contract_id,
            test_id,
            "ops/README.md should be intent/navigation only, without tutorial run sections",
            Some("ops/README.md".to_string()),
        ));
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_inv_001_inventory_completeness(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-001";
    let test_id = "ops.inventory.completeness";

    let owners_path = ctx.repo_root.join("ops/inventory/owners.json");
    let owners = read_json(&owners_path);
    let (authoritative, contract_paths) = inventory_paths(&ctx.repo_root);
    let mut violations = Vec::new();

    let mut areas = BTreeMap::new();
    if let Some(owners) = owners {
        if let Some(area_map) = owners.get("areas").and_then(|v| v.as_object()) {
            for (k, v) in area_map {
                areas.insert(k.clone(), v.clone());
            }
        }
    }

    for domain in DOMAIN_DIRS {
        let key = format!("ops/{domain}");
        if !areas.contains_key(&key) {
            violations.push(violation(
                contract_id,
                test_id,
                "domain missing ownership registration in ops/inventory/owners.json",
                Some(key),
            ));
        }
    }

    let mut files = Vec::new();
    walk_files(&ops_root(&ctx.repo_root), &mut files);
    files.sort();
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if !rel.ends_with(".json")
            || !rel.contains("policy")
            || rel.starts_with("ops/schema/")
            || rel.ends_with(".schema.json")
        {
            continue;
        }
        let referenced = authoritative.contains(&rel) || contract_paths.contains(&rel);
        if !referenced {
            violations.push(violation(
                contract_id,
                test_id,
                "policy json must be registered in inventory authoritative-file-list or contracts-map",
                Some(rel),
            ));
        }
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn is_inventory_referenced(
    rel: &str,
    authoritative: &BTreeSet<String>,
    contract_paths: &BTreeSet<String>,
) -> bool {
    if authoritative.contains(rel) || contract_paths.contains(rel) {
        return true;
    }
    for domain in DOMAIN_DIRS {
        if rel == format!("ops/{domain}/README.md") || rel == format!("ops/{domain}/CONTRACT.md") {
            return true;
        }
        if rel.starts_with(&format!("ops/{domain}/")) && path_depth(rel) == 3 {
            return true;
        }
    }
    rel == "ops/README.md"
        || rel == "ops/CONTRACT.md"
        || rel.starts_with("ops/policy/")
        || rel.starts_with("ops/schema/")
        || rel.starts_with("ops/_generated/")
        || rel.starts_with("ops/_generated.example/")
        || rel.contains("/tests/")
        || rel.contains("/fixtures/")
        || rel.contains("/generated/")
        || rel.starts_with("ops/k8s/charts/")
        || rel.starts_with("ops/k8s/values/")
        || rel.starts_with("ops/stack/kind/")
        || rel.starts_with("ops/stack/minio/")
        || rel.starts_with("ops/stack/otel/")
        || rel.starts_with("ops/stack/prometheus/")
        || rel.starts_with("ops/stack/redis/")
        || rel.starts_with("ops/stack/toxiproxy/")
        || rel.starts_with("ops/observe/alerts/")
        || rel.starts_with("ops/observe/dashboards/")
        || rel.starts_with("ops/observe/rules/")
        || rel.starts_with("ops/observe/drills/templates/")
        || rel.starts_with("ops/inventory/contracts/")
        || rel.starts_with("ops/inventory/meta/")
}

fn path_depth(rel: &str) -> usize {
    rel.split('/').count()
}

fn test_ops_inv_002_no_orphan_files(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-002";
    let test_id = "ops.inventory.no_orphan_files";
    let (authoritative, contract_paths) = inventory_paths(&ctx.repo_root);
    let mut files = Vec::new();
    walk_files(&ops_root(&ctx.repo_root), &mut files);
    files.sort();

    let mut violations = Vec::new();
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        // Scope orphan checking to governance surface paths; runtime dataset fixtures,
        // generated artifacts, and deep domain assets are validated by domain contracts.
        let depth = path_depth(&rel);
        if depth > 3 {
            continue;
        }
        if is_inventory_referenced(&rel, &authoritative, &contract_paths) {
            continue;
        }
        let ext = path.extension().and_then(|v| v.to_str()).unwrap_or("");
        if ["json", "yaml", "yml", "toml", "md"].contains(&ext) {
            violations.push(violation(
                contract_id,
                test_id,
                "ops file is not mapped by inventory references",
                Some(rel),
            ));
        }
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_inv_003_no_duplicate_ssot(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-003";
    let test_id = "ops.inventory.no_duplicate_ssot_sources";

    let mut files = Vec::new();
    walk_files(&ops_root(&ctx.repo_root), &mut files);
    files.sort();

    let mut violations = Vec::new();
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if rel.starts_with("ops/_generated/") || rel.starts_with("ops/_generated.example/") {
            continue;
        }
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if ["OWNER.md", "INDEX.md", "REQUIRED_FILES.md"].contains(&name) {
            violations.push(violation(
                contract_id,
                test_id,
                "duplicate SSOT markdown source is forbidden; use ops/inventory/**",
                Some(rel),
            ));
        }
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}
