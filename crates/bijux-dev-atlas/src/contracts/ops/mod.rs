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

mod common;

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
    common::walk_files(root, out)
}

fn rel_to_root(path: &Path, repo_root: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/")
}

fn read_json(path: &Path) -> Option<Value> {
    common::read_json(path)
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

include!("root/mod.rs");
include!("governance/mod.rs");
include!("inventory/mod.rs");
include!("datasets/mod.rs");
include!("e2e/mod.rs");
include!("environment/mod.rs");
include!("platform/mod.rs");
include!("load/mod.rs");
include!("observe/mod.rs");
include!("reporting/mod.rs");
