// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn src_root() -> PathBuf {
    crate_root().join("src")
}

fn rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        for entry in
            fs::read_dir(&path).unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
        {
            let entry = entry.expect("dir entry");
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
                continue;
            }
            if entry_path.extension().is_some_and(|ext| ext == "rs") {
                files.push(entry_path);
            }
        }
    }
    files.sort();
    files
}

fn rel(path: &Path) -> String {
    path.strip_prefix(crate_root())
        .unwrap_or(path)
        .display()
        .to_string()
}

fn assert_only_allowlisted_paths(pattern: &str, allowed_rel_paths: &[&str]) {
    let allowed: BTreeSet<&str> = allowed_rel_paths.iter().copied().collect();
    let mut offenders = Vec::new();
    for file in rust_sources(&src_root()) {
        let text =
            fs::read_to_string(&file).unwrap_or_else(|err| panic!("failed to read {}: {err}", file.display()));
        if text.contains(pattern) {
            let rp = rel(&file);
            if !allowed.contains(rp.as_str()) {
                offenders.push(rp);
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "pattern `{pattern}` found outside allowlist: {offenders:?}"
    );
}

#[test]
fn fs_calls_are_constrained_to_explicit_allowlist() {
    assert_only_allowlisted_paths(
        "std::fs::",
        &[
            "src/adapters/fs.rs",
            "src/adapters/mod.rs",
            "src/adapters/workspace_root.rs",
            "src/cli/dispatch.rs",
            "src/commands/configs.rs",
            "src/commands/docs/runtime/reference_page_generators.rs",
            "src/commands/ops/execution_runtime_mod/tests.rs",
            "src/commands/ops/execution_runtime_mod/install_status.rs",
            "src/commands/ops/runtime_mod/core_handler.rs",
            "src/commands/ops/runtime_mod/execution_handler.rs",
            "src/commands/ops/support/domain_support.rs",
            "src/commands/ops/support/manifests.rs",
            "src/commands/ops/support/tools.rs",
            "src/contracts/mod.rs",
            "src/contracts/docker/mod.rs",
            "src/contracts/ops/mod.rs",
            "src/contracts/ops/ops_extended.inc.rs",
            "src/contracts/ops/ops_domains_31_40.inc.rs",
            "src/contracts/ops/ops_registry.inc.rs",
            "src/contracts/docker/contracts_static_checks.inc.rs",
            "src/contracts/docker/contracts_tests.inc.rs",
            "src/core/checks/ops/ops/inventory_and_artifact_checks.rs",
            "src/core/ops_inventory/summary_and_fs_scan.rs",
            "src/runtime_entry.inc.rs",
            "src/runtime_entry_checks_surface.inc.rs",
        ],
    );
}

#[test]
fn process_calls_are_constrained_to_explicit_allowlist() {
    assert_only_allowlisted_paths(
        "std::process::Command",
        &[
            "src/adapters/process.rs",
            "src/adapters/world.rs",
            "src/cli/dispatch.rs",
            "src/commands/control_plane_docker_runtime_helpers.inc.rs",
            "src/commands/docs/runtime/reference_page_generators.rs",
            "src/contracts/docker/mod.rs",
            "src/runtime_entry.inc.rs",
        ],
    );
}

#[test]
fn env_var_calls_are_constrained_to_explicit_allowlist() {
    assert_only_allowlisted_paths(
        "std::env::var(",
        &["src/commands/ops/support/domain_support.rs"],
    );
}

#[test]
fn network_client_calls_are_constrained_to_explicit_allowlist() {
    assert_only_allowlisted_paths(
        "reqwest::",
        &[],
    );
    assert_only_allowlisted_paths(
        "ureq::",
        &[],
    );
}
