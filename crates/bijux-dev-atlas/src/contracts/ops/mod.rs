// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
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

fn sorted_dir_entries(root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut paths = entries
        .flatten()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn walk_files(root: &Path, out: &mut Vec<PathBuf>) {
    for path in sorted_dir_entries(root) {
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

include!("registry_validation.inc.rs");
include!("root_surface.inc.rs");
include!("inventory.inc.rs");
include!("schema.inc.rs");
include!("datasets.inc.rs");
include!("e2e.inc.rs");
include!("env.inc.rs");
include!("stack.inc.rs");
include!("k8s.inc.rs");
include!("observe.inc.rs");
include!("load.inc.rs");
include!("report.inc.rs");
include!("pillars.inc.rs");

include!("ops_root_and_inventory_contracts.inc.rs");
include!("ops_extended.inc.rs");

include!("ops_registry.inc.rs");
include!("ops_contract_docs.inc.rs");
