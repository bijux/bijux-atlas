// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn no_unwrap_or_expect_in_non_test_dev_atlas_sources() {
    let root = repo_root();
    let output = Command::new("rg")
        .current_dir(&root)
        .args([
            "-n",
            r"(?:\.unwrap\(|\.expect\()",
            "crates/bijux-dev-atlas/src",
            "-g",
            "*.rs",
            "-g",
            "!**/tests.rs",
        ])
        .output()
        .expect("run rg");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let allowed_inline_test_files: BTreeSet<&str> = [
        "crates/bijux-dev-atlas/src/adapters/mod.rs",
        "crates/bijux-dev-atlas/src/core/logging.rs",
        "crates/bijux-dev-atlas/src/core/ops_inventory/summary_and_fs_scan.rs",
        "crates/bijux-dev-atlas/src/model/mod.rs",
        "crates/bijux-dev-atlas/src/commands/ops_support/manifests.rs",
        "crates/bijux-dev-atlas/src/commands/ops_support/tools.rs",
    ]
    .into_iter()
    .collect();

    let violations: Vec<&str> = stdout
        .lines()
        .filter(|line| {
            !allowed_inline_test_files
                .iter()
                .any(|allowed| line.starts_with(allowed))
        })
        .collect();

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
