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
