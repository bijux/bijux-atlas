// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
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
fn contracts_production_sources_avoid_unwrap_and_expect() {
    let root = repo_root().join("crates/bijux-dev-atlas/src/contracts");
    let mut stack = vec![root];
    let mut offenders = Vec::new();
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read contracts dir").flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            if !name.ends_with(".rs") || name.ends_with("tests.rs") || name == "engine_tests.rs" {
                continue;
            }
            let text =
                strip_cfg_test_modules(&fs::read_to_string(&path).expect("read source file"));
            if text.contains(".unwrap(") || text.contains(".expect(") {
                offenders.push(
                    path.strip_prefix(repo_root())
                        .expect("relative path")
                        .display()
                        .to_string(),
                );
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "contracts production sources must avoid unwrap/expect:\n{}",
        offenders.join("\n")
    );
}
