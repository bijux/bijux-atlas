// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{Contract, ContractId, RunContext, TestCase, TestId, TestKind, TestResult, Violation};

const DOMAIN_DIRS: &[&str] = &[
    "datasets",
    "e2e",
    "env",
    "inventory",
    "k8s",
    "load",
    "observe",
    "report",
    "schema",
    "stack",
];

fn violation(contract_id: &str, test_id: &str, message: &str, file: Option<String>) -> Violation {
    Violation {
        contract_id: contract_id.to_string(),
        test_id: test_id.to_string(),
        file,
        line: Some(1),
        message: message.to_string(),
        evidence: None,
    }
}

fn ops_root(repo_root: &Path) -> PathBuf {
    repo_root.join("ops")
}

fn walk_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn rel_to_root(path: &Path, repo_root: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/")
}

fn read_json(path: &Path) -> Option<Value> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn markdown_line_count(path: &Path) -> usize {
    std::fs::read_to_string(path)
        .map(|c| c.lines().count())
        .unwrap_or(0)
}

fn file_sha256(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Some(format!("{:x}", hasher.finalize()))
}

fn sha256_text(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

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

fn test_ops_000_allowed_root_files(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-000";
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
    let contract_id = "OPS-000";
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
    let contract_id = "OPS-000";
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
    let contract_id = "OPS-000";
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
    let contract_id = "OPS-001";
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
    let contract_id = "OPS-001";
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
    let contract_id = "OPS-001";
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
    let contract_id = "OPS-002";
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
    let contract_id = "OPS-002";
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
    let contract_id = "OPS-003";
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
    let contract_id = "OPS-003";
    let test_id = "ops.markdown_budget.contract";
    let mut violations = Vec::new();

    let root_contract = ctx.repo_root.join("ops/CONTRACT.md");
    let lines = markdown_line_count(&root_contract);
    if lines > 400 {
        violations.push(violation(
            contract_id,
            test_id,
            "ops/CONTRACT.md exceeds line budget (max 400)",
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
    let contract_id = "OPS-004";
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

include!("ops_extended.inc.rs");

include!("ops_registry.inc.rs");

fn test_ops_root_010_forbid_deleted_doc_names(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-010";
    let test_id = "ops.root.forbid_deleted_doc_names";
    let mut files = Vec::new();
    walk_files(&ops_root(&ctx.repo_root), &mut files);
    files.sort();

    let forbidden = BTreeSet::from([
        "ARTIFACTS.md",
        "DRIFT.md",
        "NAMING.md",
        "INDEX.md",
        "OWNER.md",
        "REQUIRED_FILES.md",
        "DIRECTORY_BUDGET_POLICY.md",
        "GENERATED_LIFECYCLE.md",
    ]);

    let mut violations = Vec::new();
    for path in files {
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if forbidden.contains(name) {
            violations.push(violation(
                contract_id,
                test_id,
                "forbidden legacy ops markdown document reintroduced",
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

fn test_ops_contract_doc_generated_match(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-005";
    let test_id = "ops.contract_doc.generated_match";
    let expected = match render_contract_markdown(&ctx.repo_root) {
        Ok(text) => text,
        Err(err) => return TestResult::Error(err),
    };
    let path = ctx.repo_root.join("ops/CONTRACT.md");
    let actual = std::fs::read_to_string(&path).unwrap_or_default();
    if actual == expected {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "ops/CONTRACT.md drifted from generated contract registry",
            Some("ops/CONTRACT.md".to_string()),
        )])
    }
}

fn has_policy_keyword(content: &str) -> bool {
    let lower = content.to_ascii_lowercase();
    lower.contains("must ") || lower.contains("shall ") || lower.contains("forbidden")
}

fn has_ops_contract_id(content: &str) -> bool {
    content.contains("OPS-")
}

fn test_ops_docs_001_policy_keyword_requires_contract_id(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-DOCS-001";
    let test_id = "ops.docs.policy_keyword_requires_contract_id";
    let root = ctx.repo_root.join("docs/operations");
    let Ok(entries) = std::fs::read_dir(&root) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "docs/operations directory is missing",
            Some("docs/operations".to_string()),
        )]);
    };

    let mut violations = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if !path
            .extension()
            .and_then(|v| v.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        if has_policy_keyword(&content) && !has_ops_contract_id(&content) {
            violations.push(violation(
                contract_id,
                test_id,
                "operations doc declares policy keywords without OPS contract id reference",
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

fn test_ops_docs_002_index_crosslinks_contracts(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-DOCS-002";
    let test_id = "ops.docs.index_crosslinks_contracts";
    let path = ctx.repo_root.join("docs/operations/INDEX.md");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "docs/operations/INDEX.md is missing",
            Some("docs/operations/INDEX.md".to_string()),
        )]);
    };
    let has_boundary = content.contains("Operational policies are enforced by contracts");
    let has_contract_ref = content.contains("OPS-");
    if has_boundary && has_contract_ref {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "docs/operations/INDEX.md must state docs/contracts boundary and include OPS contract references",
            Some("docs/operations/INDEX.md".to_string()),
        )])
    }
}

pub fn render_contract_markdown(repo_root: &Path) -> Result<String, String> {
    let rows = contracts(repo_root)?;
    let mut out = String::new();
    out.push_str("# Ops Contract\n\n");
    out.push_str("- Owner: `bijux-atlas-operations`\n");
    out.push_str("- Enforced by: `bijux dev atlas contracts ops`\n\n");
    out.push_str("## Contract Registry\n\n");
    for contract in &rows {
        out.push_str(&format!("### {} {}\n\n", contract.id.0, contract.title));
        out.push_str("Tests:\n");
        for case in &contract.tests {
            let mode = match case.kind {
                TestKind::Pure => "static",
                TestKind::Subprocess | TestKind::Network => "effect",
            };
            out.push_str(&format!(
                "- `{}` ({mode}, {:?}): {}\n",
                case.id.0, case.kind, case.title
            ));
        }
        out.push('\n');
    }
    out.push_str("## Rule\n\n");
    out.push_str("- Contract ID or test ID missing from this document means it does not exist.\n");
    Ok(out)
}

pub fn sync_contract_markdown(repo_root: &Path) -> Result<(), String> {
    let rendered = render_contract_markdown(repo_root)?;
    let path = repo_root.join("ops/CONTRACT.md");
    std::fs::write(&path, rendered).map_err(|e| format!("write {} failed: {e}", path.display()))
}
