// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collect_rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, out);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                out.push(path);
            }
        }
    }
}

fn ignored_test_files() -> BTreeSet<&'static str> {
    [
        "src/main_tests.rs",
        "src/core/lib_tests.rs",
        "src/ops_runtime_execution/runtime_mod/tests.rs",
    ]
    .into_iter()
    .collect()
}

fn staged_effect_exceptions() -> BTreeSet<&'static str> {
    [
        // End-state target: effects live under adapters; current convergence still has legacy command/core sites.
        "src/adapters/mod.rs",
        "src/adapters/fs.rs",
        "src/adapters/process.rs",
        "src/adapters/world.rs",
        "src/main.rs",
        "src/ops_support.rs",
        "src/commands/ops_support/manifests.rs",
        "src/commands/ops_support/tools.rs",
        "src/commands/docs_runtime/command_dispatch.rs",
        "src/ops_commands/runtime_mod/core_handler.rs",
        "src/ops_commands/runtime_mod/execution_handler.rs",
        "src/ops_runtime_execution/runtime_mod/install_status.rs",
        // Imported modules pending ports/adapters rewiring.
        "src/core/mod.rs",
        "src/core/checks/ops.rs",
        "src/core/checks/ops/ops/inventory_and_artifact_checks.rs",
        "src/core/ops_inventory/types_and_manifests.rs",
        "src/core/ops_inventory/summary_and_fs_scan.rs",
        "src/policies/validate.rs",
    ]
    .into_iter()
    .collect()
}

fn files_with_effect_pattern(patterns: &[&str]) -> Vec<String> {
    let root = crate_root();
    let src_root = root.join("src");
    let ignored_tests = ignored_test_files();
    let exceptions = staged_effect_exceptions();

    let mut files = Vec::new();
    collect_rs_files(&src_root, &mut files);
    files.sort();

    let mut violations = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .expect("under crate root")
            .to_string_lossy()
            .replace('\\', "/");
        if ignored_tests.contains(rel.as_str()) {
            continue;
        }
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        let has_effect = patterns.iter().any(|pattern| content.contains(pattern));
        if has_effect && !exceptions.contains(rel.as_str()) {
            violations.push(rel);
        }
    }

    violations
}

#[test]
fn no_std_fs_outside_adapters_staged_exceptions() {
    let violations = files_with_effect_pattern(&["std::fs::"]);
    assert!(
        violations.is_empty(),
        "unexpected std::fs usage: {violations:?}"
    );
}

#[test]
fn no_process_command_outside_adapters_staged_exceptions() {
    let violations = files_with_effect_pattern(&["std::process::Command"]);
    assert!(
        violations.is_empty(),
        "unexpected std::process::Command usage: {violations:?}"
    );
}

#[test]
fn no_env_var_or_current_dir_outside_adapters_staged_exceptions() {
    let violations = files_with_effect_pattern(&["std::env::var(", "std::env::current_dir("]);
    assert!(
        violations.is_empty(),
        "unexpected std::env host lookup usage: {violations:?}"
    );
}
