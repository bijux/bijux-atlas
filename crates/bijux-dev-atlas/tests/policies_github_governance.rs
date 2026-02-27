// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn collect_files_recursive(root: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files_recursive(&path, out);
            } else {
                out.push(path);
            }
        }
    }
}

#[test]
fn workflows_do_not_invoke_scripts_directly() {
    let root = workspace_root();
    let workflows = root.join(".github/workflows");
    let mut files = Vec::new();
    collect_files_recursive(&workflows, &mut files);
    files.sort();

    let mut violations = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .expect("path must be under workspace root")
            .to_string_lossy()
            .to_string();
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        for line in content.lines() {
            let trimmed = line.trim();
            let is_direct_script_runner =
                trimmed.contains("bash scripts/") || trimmed.contains("python scripts/");
            let is_direct_python3_script = trimmed.contains("python3 scripts/");
            if is_direct_script_runner || is_direct_python3_script {
                violations.push(format!("{rel}: {trimmed}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "workflows must not invoke scripts directly: {violations:?}"
    );
}

#[test]
fn release_candidate_workflow_requires_ops_readiness_gate() {
    let root = workspace_root();
    let workflow = root.join(".github/workflows/release-candidate.yml");
    let content = fs::read_to_string(&workflow).expect("read release-candidate workflow");
    assert!(
        content.contains("ops report readiness --format json"),
        "release-candidate workflow must execute `ops report readiness --format json`"
    );
    assert!(
        content.contains("ops-readiness.json"),
        "release-candidate workflow must persist ops readiness report artifact"
    );
}
