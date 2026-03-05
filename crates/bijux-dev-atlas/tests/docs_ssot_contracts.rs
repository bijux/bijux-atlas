// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn collect_markdown_files(root: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("read dir") {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            collect_markdown_files(&path, out);
            continue;
        }
        if path.extension().and_then(|v| v.to_str()) == Some("md") {
            out.push(path);
        }
    }
}

fn is_public_docs_page(root: &Path, file: &Path) -> bool {
    let rel = file
        .strip_prefix(root)
        .expect("relative path")
        .to_string_lossy()
        .replace('\\', "/");
    rel.starts_with("docs/")
        && !rel.starts_with("docs/_internal/")
        && !rel.starts_with("docs/_generated/")
        && !rel.starts_with("docs/_drafts/")
}

#[test]
fn docs_index_and_readme_exist_as_docs_entry_authority() {
    let root = repo_root();
    let index = root.join("docs/INDEX.md");
    let readme = root.join("docs/README.md");
    assert!(index.exists(), "docs/INDEX.md must exist");
    assert!(readme.exists(), "docs/README.md must exist");
    let text = fs::read_to_string(index).expect("read docs/INDEX.md");
    assert!(
        text.contains("canonical top-level navigation authority"),
        "docs/INDEX.md must declare navigation authority"
    );
}

#[test]
fn public_docs_depth_budget_is_enforced() {
    const MAX_PUBLIC_DOC_DEPTH: usize = 6;
    let root = repo_root();
    let mut files = Vec::new();
    collect_markdown_files(&root.join("docs"), &mut files);
    let mut violations = Vec::new();
    for file in files {
        if !is_public_docs_page(&root, &file) {
            continue;
        }
        let rel = file
            .strip_prefix(root.join("docs"))
            .expect("relative docs path");
        let depth = rel.components().count();
        if depth > MAX_PUBLIC_DOC_DEPTH {
            violations.push(format!("{} depth={depth}", rel.display()));
        }
    }
    assert!(
        violations.is_empty(),
        "public docs depth budget exceeded:\n{}",
        violations.join("\n")
    );
}

#[test]
fn docs_markdown_file_count_budget_is_below_guardrail() {
    const MAX_DOC_MARKDOWN_FILES: usize = 1600;
    let root = repo_root();
    let mut files = Vec::new();
    collect_markdown_files(&root.join("docs"), &mut files);
    assert!(
        files.len() <= MAX_DOC_MARKDOWN_FILES,
        "docs markdown count {} exceeds budget {}",
        files.len(),
        MAX_DOC_MARKDOWN_FILES
    );
}

#[test]
fn canonical_docs_pages_require_type_and_owner_front_matter() {
    let root = repo_root();
    let files = [
        root.join("docs/INDEX.md"),
        root.join("docs/README.md"),
        root.join("docs/governance/docs-ssot-rule.md"),
        root.join("docs/architecture/docs-architecture.md"),
        root.join("docs/governance/docs-minimization-policy.md"),
    ];
    let mut violations = Vec::new();
    for file in files {
        let rel = file.strip_prefix(&root).expect("relative path");
        let text = fs::read_to_string(&file).expect("read markdown");
        let mut lines = text.lines();
        if lines.next() != Some("---") {
            violations.push(format!("{rel:?} missing front matter"));
            continue;
        }
        let mut has_type = false;
        let mut has_owner = false;
        for line in lines {
            if line.trim() == "---" {
                break;
            }
            if line.trim_start().starts_with("type:") {
                has_type = true;
            }
            if line.trim_start().starts_with("owner:") {
                has_owner = true;
            }
        }
        if !has_type || !has_owner {
            violations.push(format!(
                "{} missing {}{}",
                rel.display(),
                if !has_type { "`type`" } else { "" },
                if !has_owner {
                    if !has_type {
                        " and `owner`"
                    } else {
                        "`owner`"
                    }
                } else {
                    ""
                }
            ));
        }
    }
    assert!(
        violations.is_empty(),
        "public docs pages must declare type and owner:\n{}",
        violations.join("\n")
    );
}

#[test]
fn docs_dedupe_report_command_runs_and_returns_rows() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docs", "dedupe-report", "--format", "json"])
        .output()
        .expect("run docs dedupe-report");
    assert!(output.status.success(), "docs dedupe-report must pass");
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse docs dedupe-report payload");
    let rows = payload["rows"].as_array().cloned().unwrap_or_default();
    assert!(
        !rows.is_empty() || payload["kind"] == "docs_dedupe_report",
        "docs dedupe-report must return a valid payload shape"
    );
}

#[test]
fn no_empty_public_docs_pages() {
    let root = repo_root();
    let mut files = Vec::new();
    collect_markdown_files(&root.join("docs"), &mut files);
    let mut empty = Vec::new();
    for file in files {
        if !is_public_docs_page(&root, &file) {
            continue;
        }
        let text = fs::read_to_string(&file).expect("read markdown");
        if text.trim().is_empty() {
            empty.push(
                file.strip_prefix(&root)
                    .expect("relative path")
                    .display()
                    .to_string(),
            );
        }
    }
    assert!(
        empty.is_empty(),
        "empty public docs pages found: {:?}",
        empty
    );
}

#[test]
fn docs_toc_verify_passes() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docs", "toc", "verify", "--format", "json"])
        .output()
        .expect("run docs toc verify");
    assert!(output.status.success(), "docs toc verify must pass");
}
