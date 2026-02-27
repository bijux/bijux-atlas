// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::{
    Contract, ContractId, RunContext, TestCase, TestId, TestKind, TestResult, Violation,
};

const OPS_ROOT_CONTRACT_ID: &str = "OPS-000";
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

fn violation(test_id: &str, message: &str, file: Option<String>) -> Violation {
    Violation {
        contract_id: OPS_ROOT_CONTRACT_ID.to_string(),
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

fn rel_to_ops(path: &Path, repo_root: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn test_allowed_root_files(ctx: &RunContext) -> TestResult {
    let root = ops_root(&ctx.repo_root);
    let Ok(entries) = std::fs::read_dir(&root) else {
        return TestResult::Fail(vec![violation(
            "ops.dir.allowed_root_files",
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
                    "ops.dir.allowed_root_files",
                    "unexpected root file under ops/",
                    Some(rel_to_ops(&path, &ctx.repo_root)),
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

fn test_forbid_extra_markdown_root(ctx: &RunContext) -> TestResult {
    let root = ops_root(&ctx.repo_root);
    let Ok(entries) = std::fs::read_dir(&root) else {
        return TestResult::Fail(vec![violation(
            "ops.dir.forbid_extra_markdown_root",
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
                "ops.dir.forbid_extra_markdown_root",
                "only ops/README.md and ops/CONTRACT.md are allowed at ops root",
                Some(rel_to_ops(&path, &ctx.repo_root)),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_allow_only_known_domain_dirs(ctx: &RunContext) -> TestResult {
    let root = ops_root(&ctx.repo_root);
    let Ok(entries) = std::fs::read_dir(&root) else {
        return TestResult::Fail(vec![violation(
            "ops.dir.allow_only_known_domain_dirs",
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
                "ops.dir.allow_only_known_domain_dirs",
                "unknown directory under ops root",
                Some(rel_to_ops(&path, &ctx.repo_root)),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_forbid_extra_markdown_recursive(ctx: &RunContext) -> TestResult {
    let ops_root = ops_root(&ctx.repo_root);
    let mut files = Vec::new();
    walk_files(&ops_root, &mut files);
    files.sort();

    let allowed_md = {
        let mut set = BTreeSet::new();
        set.insert("ops/README.md".to_string());
        set.insert("ops/CONTRACT.md".to_string());
        for domain in DOMAIN_DIRS {
            set.insert(format!("ops/{domain}/README.md"));
            set.insert(format!("ops/{domain}/CONTRACT.md"));
        }
        set
    };

    let mut violations = Vec::new();
    for path in files {
        let rel = rel_to_ops(&path, &ctx.repo_root).replace('\\', "/");
        if rel.starts_with("ops/_generated/") || rel.starts_with("ops/_generated.example/") {
            continue;
        }
        let is_markdown = path
            .extension()
            .and_then(|v| v.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"));
        if is_markdown && !allowed_md.contains(&rel) {
            violations.push(violation(
                "ops.dir.forbid_extra_markdown_recursive",
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

pub fn contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
    Ok(vec![Contract {
        id: ContractId(OPS_ROOT_CONTRACT_ID.to_string()),
        title: "ops directory contract",
        tests: vec![
            TestCase {
                id: TestId("ops.dir.allowed_root_files".to_string()),
                title: "ops root allows only contract/readme root files",
                kind: TestKind::Pure,
                run: test_allowed_root_files,
            },
            TestCase {
                id: TestId("ops.dir.forbid_extra_markdown_root".to_string()),
                title: "ops root forbids extra markdown",
                kind: TestKind::Pure,
                run: test_forbid_extra_markdown_root,
            },
            TestCase {
                id: TestId("ops.dir.allow_only_known_domain_dirs".to_string()),
                title: "ops root allows only canonical domain directories",
                kind: TestKind::Pure,
                run: test_allow_only_known_domain_dirs,
            },
            TestCase {
                id: TestId("ops.dir.forbid_extra_markdown_recursive".to_string()),
                title: "ops tree forbids markdown outside approved domain docs",
                kind: TestKind::Pure,
                run: test_forbid_extra_markdown_recursive,
            },
        ],
    }])
}

pub fn contract_explain(contract_id: &str) -> String {
    match contract_id {
        OPS_ROOT_CONTRACT_ID => {
            "Restrict ops/ to canonical root files and canonical domain surfaces. Move policy text into executable contracts and keep docs minimal."
                .to_string()
        }
        _ => "Fix violations listed for this contract and rerun `bijux dev atlas contracts ops`."
            .to_string(),
    }
}
