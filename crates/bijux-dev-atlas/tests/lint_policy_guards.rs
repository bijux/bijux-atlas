// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn strip_cfg_test_modules(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.trim() == "#[cfg(test)]" {
            let Some(next_line) = lines.next() else {
                break;
            };
            if next_line.contains("mod tests") {
                let mut brace_depth = next_line.matches('{').count();
                brace_depth = brace_depth.saturating_sub(next_line.matches('}').count());
                while brace_depth > 0 {
                    let Some(test_line) = lines.next() else {
                        break;
                    };
                    brace_depth += test_line.matches('{').count();
                    brace_depth = brace_depth.saturating_sub(test_line.matches('}').count());
                }
                continue;
            }
            out.push_str(line);
            out.push('\n');
            out.push_str(next_line);
            out.push('\n');
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

#[test]
fn no_unwrap_or_expect_in_non_test_dev_atlas_sources() {
    let root = repo_root().join("crates/bijux-dev-atlas/src");
    let allowed_inline_test_files: BTreeSet<&str> = [
        "crates/bijux-dev-atlas/src/adapters/mod.rs",
        "crates/bijux-dev-atlas/src/core/logging.rs",
        "crates/bijux-dev-atlas/src/core/ops_inventory/summary_and_fs_scan.rs",
        "crates/bijux-dev-atlas/src/model/mod.rs",
        "crates/bijux-dev-atlas/src/commands/ops/support/manifests.rs",
        "crates/bijux-dev-atlas/src/commands/ops/support/tools.rs",
        "crates/bijux-dev-atlas/src/commands/system.rs",
        "crates/bijux-dev-atlas/src/schema_support.rs",
        "crates/bijux-dev-atlas/src/contracts/mod.rs",
        "crates/bijux-dev-atlas/src/contracts/engine_tests.rs",
        "crates/bijux-dev-atlas/src/contracts/docker/contracts_tests.rs",
    ]
    .into_iter()
    .collect();

    let mut stack = vec![root.clone()];
    let mut violations = Vec::new();
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read source dir").flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let Some(relative_path) = path
                .strip_prefix(repo_root())
                .ok()
                .and_then(|value| value.to_str())
            else {
                continue;
            };
            if !relative_path.ends_with(".rs") || relative_path.ends_with("tests.rs") {
                continue;
            }
            if allowed_inline_test_files.contains(relative_path) {
                continue;
            }
            let text =
                strip_cfg_test_modules(&fs::read_to_string(&path).expect("read source file"));
            for (line_number, line) in text.lines().enumerate() {
                if line.contains(".unwrap(") || line.contains(".expect(") {
                    violations.push(format!("{relative_path}:{}:{line}", line_number + 1));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "unexpected unwrap/expect in non-test source:\n{}",
        violations.join("\n")
    );
}

#[test]
fn no_todo_macro_anywhere_in_dev_atlas_crate() {
    let root = repo_root();
    let output = Command::new("rg")
        .current_dir(&root)
        .args(["-n", r"\btodo!\(", "crates/bijux-dev-atlas"])
        .output()
        .expect("run rg");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim().is_empty(),
        "todo! macro is forbidden:\n{stdout}"
    );
}
