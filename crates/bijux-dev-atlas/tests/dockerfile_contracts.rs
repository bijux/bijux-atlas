// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

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
    let policy_path = root.join("docker/contracts/digest-pinning.json");
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
