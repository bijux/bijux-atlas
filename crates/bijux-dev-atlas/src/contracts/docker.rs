// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use super::{
    Contract, ContractId, ContractRegistry, RunContext, TestCase, TestId, TestKind, TestResult,
    Violation,
};

fn all_dockerfiles(repo_root: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut files = Vec::new();
    let root = repo_root.join("docker/images");
    if !root.exists() {
        return Ok(files);
    }
    let mut queue = std::collections::VecDeque::from([root]);
    while let Some(dir) = queue.pop_front() {
        for entry in std::fs::read_dir(&dir).map_err(|e| format!("read_dir failed: {e}"))? {
            let entry = entry.map_err(|e| format!("read_dir entry failed: {e}"))?;
            let path = entry.path();
            if path.is_dir() {
                queue.push_back(path);
                continue;
            }
            if path
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s == "Dockerfile")
            {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn parse_from(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with("FROM ") {
        return None;
    }
    let tokens = trimmed.split_whitespace().collect::<Vec<_>>();
    if tokens.len() < 2 {
        return None;
    }
    let img = if tokens[1] == "--platform=$BUILDPLATFORM" && tokens.len() >= 3 {
        tokens[2]
    } else {
        tokens[1]
    };
    Some(img.to_string())
}

fn test_no_latest(ctx: &RunContext) -> TestResult {
    let files = match all_dockerfiles(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        let text = match std::fs::read_to_string(&file) {
            Ok(v) => v,
            Err(e) => return TestResult::Error(format!("read {} failed: {e}", file.display())),
        };
        for (idx, line) in text.lines().enumerate() {
            let Some(from) = parse_from(line) else {
                continue;
            };
            if from.ends_with(":latest") || from == "latest" {
                violations.push(Violation {
                    contract_id: "DOCKER-001".to_string(),
                    test_id: "docker.from.no_latest".to_string(),
                    file: Some(rel.clone()),
                    line: Some(idx + 1),
                    message: "latest tag is forbidden".to_string(),
                    evidence: Some(from),
                });
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_digest_required(ctx: &RunContext) -> TestResult {
    let files = match all_dockerfiles(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        let text = match std::fs::read_to_string(&file) {
            Ok(v) => v,
            Err(e) => return TestResult::Error(format!("read {} failed: {e}", file.display())),
        };
        for (idx, line) in text.lines().enumerate() {
            let Some(from) = parse_from(line) else {
                continue;
            };
            if !from.contains("@sha256:") {
                violations.push(Violation {
                    contract_id: "DOCKER-002".to_string(),
                    test_id: "docker.from.digest_required".to_string(),
                    file: Some(rel.clone()),
                    line: Some(idx + 1),
                    message: "base image must be digest-pinned".to_string(),
                    evidence: Some(from),
                });
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

pub fn contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
    Ok(vec![
        Contract {
            id: ContractId("DOCKER-001".to_string()),
            title: "no latest tags",
            tests: vec![TestCase {
                id: TestId("docker.from.no_latest".to_string()),
                title: "forbid latest tag in FROM",
                kind: TestKind::Pure,
                run: test_no_latest,
            }],
        },
        Contract {
            id: ContractId("DOCKER-002".to_string()),
            title: "digest pins required",
            tests: vec![TestCase {
                id: TestId("docker.from.digest_required".to_string()),
                title: "require digest-pinned FROM images",
                kind: TestKind::Pure,
                run: test_digest_required,
            }],
        },
    ])
}

pub struct DockerContractRegistry;

impl ContractRegistry for DockerContractRegistry {
    fn contracts(repo_root: &Path) -> Result<Vec<Contract>, String> {
        contracts(repo_root)
    }
}
