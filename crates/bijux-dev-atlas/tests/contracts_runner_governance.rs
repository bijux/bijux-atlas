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

#[test]
fn contracts_runtime_docs_define_stable_exit_codes() {
    let text = fs::read_to_string(repo_root().join("docs/control-plane/contracts.md"))
        .expect("read contracts control plane doc");
    for expected in [
        "- `0`: all selected contracts passed.",
        "- `1`: one or more non-required contracts failed.",
        "- `2`: usage error, including invalid wildcard filters or missing required flags.",
        "- `3`: internal runner error.",
        "- `4`: one or more required contracts failed.",
    ] {
        assert!(
            text.contains(expected),
            "contracts control-plane doc missing exit code line `{expected}`"
        );
    }
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
            let text = fs::read_to_string(&path).expect("read source file");
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
