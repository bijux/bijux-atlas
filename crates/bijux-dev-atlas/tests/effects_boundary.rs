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
        "src/commands/ops/execution_runtime_mod/tests.rs",
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
        "src/adapters/workspace_root.rs",
        "src/adapters/world.rs",
        "src/main.rs",
        "src/cli/dispatch.rs",
        "src/commands/configs.rs",
        "src/commands/control_plane_docker_runtime_helpers.rs",
        "src/commands/ops/support/manifests.rs",
        "src/commands/ops/support/tools.rs",
        "src/commands/ops/support/domain_support.rs",
        "src/commands/docs/runtime/command_dispatch.rs",
        "src/commands/docs/runtime/reference_page_generators.rs",
        "src/contracts/mod.rs",
        "src/contracts/docker/mod.rs",
        "src/contracts/ops/mod.rs",
        "src/contracts/ops/datasets/lifecycle_contracts.rs",
        "src/contracts/ops/platform/shared_static_contracts.rs",
        "src/contracts/ops/platform/verification_effect_contracts.rs",
        "src/contracts/ops/governance/registry_catalog.rs",
        "src/contracts/docker/contracts_static_checks.rs",
        "src/contracts/docker/contracts_tests.rs",
        "src/commands/ops/runtime_mod/core_handler.rs",
        "src/commands/ops/runtime_mod/execution_handler.rs",
        "src/commands/ops/execution_runtime_mod/install_status.rs",
        "src/runtime_entry.rs",
        "src/runtime_entry_checks_surface.rs",
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

fn staged_stdio_exceptions() -> BTreeSet<&'static str> {
    [
        "src/main.rs",
        "src/cli/dispatch.rs",
        "src/commands/build.rs",
        "src/commands/configs.rs",
        "src/commands/control_plane.rs",
        "src/commands/docs/runtime/docs_command_router.rs",
        "src/commands/ops/runtime.rs",
    ]
    .into_iter()
    .collect()
}

fn staged_time_random_exceptions() -> BTreeSet<&'static str> {
    [
        "src/adapters/mod.rs",
        "src/adapters/fs.rs",
        "src/adapters/workspace_root.rs",
        "src/ports/mod.rs",
    ]
    .into_iter()
    .collect()
}

fn files_with_pattern(patterns: &[&str], exceptions: &BTreeSet<&'static str>) -> Vec<String> {
    let root = crate_root();
    let src_root = root.join("src");
    let ignored_tests = ignored_test_files();

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
    let violations = files_with_pattern(&["std::fs::"], &staged_effect_exceptions());
    assert!(
        violations.is_empty(),
        "unexpected std::fs usage: {violations:?}"
    );
}

#[test]
fn no_process_command_outside_adapters_staged_exceptions() {
    let violations = files_with_pattern(&["std::process::Command"], &staged_effect_exceptions());
    assert!(
        violations.is_empty(),
        "unexpected std::process::Command usage: {violations:?}"
    );
}

#[test]
fn no_env_var_or_current_dir_outside_adapters_staged_exceptions() {
    let violations = files_with_pattern(
        &["std::env::var(", "std::env::current_dir("],
        &staged_effect_exceptions(),
    );
    assert!(
        violations.is_empty(),
        "unexpected std::env host lookup usage: {violations:?}"
    );
}

#[test]
fn no_reqwest_outside_adapters_staged_exceptions() {
    let violations = files_with_pattern(&["reqwest::"], &staged_effect_exceptions());
    assert!(
        violations.is_empty(),
        "unexpected reqwest usage outside adapters: {violations:?}"
    );
}

#[test]
fn no_print_macros_outside_cli_and_commands_staged_exceptions() {
    let violations = files_with_pattern(&["println!(", "eprintln!("], &staged_stdio_exceptions());
    assert!(
        violations.is_empty(),
        "unexpected stdio macro usage outside CLI/commands staged exceptions: {violations:?}"
    );
}

#[test]
fn no_direct_time_apis_outside_ports_and_adapters_staged_exceptions() {
    let violations = files_with_pattern(
        &[
            "SystemTime::now(",
            "UNIX_EPOCH",
            "std::time::SystemTime::now(",
        ],
        &staged_time_random_exceptions(),
    );
    assert!(
        violations.is_empty(),
        "unexpected direct time api usage outside ports/adapters staged exceptions: {violations:?}"
    );
}

#[test]
fn no_randomness_apis_in_dev_atlas_sources() {
    let violations = files_with_pattern(
        &["rand::", "thread_rng(", "StdRng", "SmallRng", "OsRng"],
        &BTreeSet::new(),
    );
    assert!(
        violations.is_empty(),
        "unexpected randomness api usage in dev-atlas sources: {violations:?}"
    );
}
