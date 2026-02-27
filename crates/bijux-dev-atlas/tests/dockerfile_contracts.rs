// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd as _;
use bijux_dev_atlas as _;
use clap as _;
use criterion as _;
use regex as _;
use serde as _;
use serde_yaml as _;
use sha2 as _;
use tempfile as _;
use toml as _;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn extract_copy_sources(line: &str) -> Option<Vec<String>> {
    let trimmed = line.trim();
    if !trimmed.starts_with("COPY ") {
        return None;
    }
    if trimmed.contains("--from=") {
        return None;
    }
    let rest = trimmed.trim_start_matches("COPY ").trim();
    let tokens: Vec<&str> = rest.split_whitespace().collect();
    if tokens.len() < 2 {
        return None;
    }
    let srcs = tokens[..tokens.len() - 1]
        .iter()
        .map(|s| s.trim_matches('"').to_string())
        .collect::<Vec<_>>();
    Some(srcs)
}

#[test]
fn runtime_dockerfile_copy_paths_exist() {
    let root = workspace_root();
    let dockerfile = root.join("docker/images/runtime/Dockerfile");
    let content = fs::read_to_string(&dockerfile).expect("read dockerfile");

    let mut missing = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        let Some(srcs) = extract_copy_sources(line) else {
            continue;
        };
        for src in srcs {
            let path = Path::new(&src);
            if src == "." || src.starts_with('/') {
                continue;
            }
            if !root.join(path).exists() {
                missing.push(format!("{}:{} -> {}", dockerfile.display(), idx + 1, src));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "runtime Dockerfile COPY sources must exist:\n{}",
        missing.join("\n")
    );
}

#[test]
fn runtime_dockerfile_base_images_follow_digest_policy() {
    let root = workspace_root();
    let dockerfile = root.join("docker/images/runtime/Dockerfile");
    let policy_path = root.join("docker/policy.json");
    let docker_text = fs::read_to_string(&dockerfile).expect("read dockerfile");
    let policy_text = fs::read_to_string(&policy_path).expect("read digest policy");
    let policy: serde_json::Value = serde_json::from_str(&policy_text).expect("policy json");
    let exceptions: Vec<String> = policy["allow_tagged_images_exceptions"]
        .as_array()
        .expect("allow_tagged_images_exceptions")
        .iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect();

    let mut violations = Vec::new();
    for (idx, raw) in docker_text.lines().enumerate() {
        let line = raw.trim();
        if !line.starts_with("FROM ") {
            continue;
        }
        let from_spec = line
            .split_whitespace()
            .nth(1)
            .expect("FROM image")
            .to_string();
        let is_digest_pinned = from_spec.contains("@sha256:");
        let is_exception = exceptions.iter().any(|e| e == &from_spec);
        let uses_latest = from_spec.ends_with(":latest") || from_spec == "latest";
        if uses_latest {
            violations.push(format!(
                "{}:{} uses forbidden latest tag `{from_spec}`",
                dockerfile.display(),
                idx + 1
            ));
            continue;
        }
        if !is_digest_pinned && !is_exception {
            violations.push(format!(
                "{}:{} image `{from_spec}` must be digest pinned or allowlisted exception",
                dockerfile.display(),
                idx + 1
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "runtime Dockerfile base image policy violations:\n{}",
        violations.join("\n")
    );
}

#[test]
fn runtime_dockerfile_has_no_unapproved_network_build_steps() {
    let root = workspace_root();
    let dockerfile = root.join("docker/images/runtime/Dockerfile");
    let text = fs::read_to_string(&dockerfile).expect("read dockerfile");
    let policy_path = root.join("docker/policy.json");
    let policy_text = fs::read_to_string(&policy_path).expect("read docker policy");
    let policy: serde_json::Value = serde_json::from_str(&policy_text).expect("policy json");
    let disallowed = policy["build_network_policy"]["forbidden_tokens"]
        .as_array()
        .expect("forbidden_tokens")
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>();
    let mut violations = Vec::new();
    for (idx, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if !line.starts_with("RUN ") {
            continue;
        }
        for token in &disallowed {
            if line.contains(token) {
                violations.push(format!(
                    "{}:{} uses disallowed networked build token `{}` in `{}`",
                    dockerfile.display(),
                    idx + 1,
                    token.trim(),
                    line
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "runtime Dockerfile network policy violations:\n{}",
        violations.join("\n")
    );
}

#[test]
fn static_contract_runner_reports_runtime_dockerfile_path_on_failure() {
    let root = workspace_root();
    let report = bijux_dev_atlas::contracts::run(
        "docker",
        bijux_dev_atlas::contracts::docker::contracts,
        &root,
        &bijux_dev_atlas::contracts::RunOptions {
            mode: bijux_dev_atlas::contracts::Mode::Static,
            allow_subprocess: false,
            allow_network: false,
            fail_fast: false,
            contract_filter: Some("DOCKER-006".to_string()),
            test_filter: Some("docker.from.no_latest".to_string()),
            list_only: false,
            artifacts_root: None,
        },
    )
    .expect("run docker contracts");
    let payload = bijux_dev_atlas::contracts::to_json(&report);
    let tests = payload["tests"].as_array().expect("tests array");
    assert!(
        tests
            .iter()
            .all(|row| row["contract_id"].as_str() == Some("DOCKER-006"))
    );
}
