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
        for entry in fs::read_dir(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
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
        let text = fs::read_to_string(&file)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", file.display()));
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
            "src/contracts/engine_runner.rs",
            "src/contracts/docker/mod.rs",
            "src/contracts/docker/docker_static_build_checks.rs",
            "src/contracts/docker/docker_static_dockerfile_policy_checks.rs",
            "src/contracts/docker/docker_static_registry_policy_checks.rs",
            "src/contracts/docker/docker_static_support.rs",
            "src/contracts/ops/mod.rs",
            "src/contracts/make/mod.rs",
            "src/contracts/ops/datasets/fixture_contracts.rs",
            "src/contracts/ops/datasets/lifecycle_contracts.rs",
            "src/contracts/ops/e2e/input_contracts.rs",
            "src/contracts/ops/observe/effect_contracts.rs",
            "src/contracts/ops/platform/kubernetes_effect_contracts.rs",
            "src/contracts/ops/platform/kubernetes_support_contracts.rs",
            "src/contracts/ops/platform/shared_static_contracts.rs",
            "src/contracts/ops/platform/verification_effect_contracts.rs",
            "src/contracts/ops/governance/registry_catalog.rs",
            "src/contracts/docker/contracts_static_checks.rs",
            "src/contracts/docker/contracts_tests.rs",
            "src/contracts/ops/reporting/documentation_contracts.rs",
            "src/contracts/ops/reporting/markdown_contracts.rs",
            "src/contracts/ops/reporting/stack_release_contracts.rs",
            "src/contracts/ops/root/root_contracts.rs",
            "src/contracts/configs/configs_registry_indexing.rs",
            "src/contracts/configs/configs_registry_model.rs",
            "src/contracts/configs/configs_surface_contracts.rs",
            "src/contracts/docs/contracts_link_checks.inc.rs",
            "src/contracts/docs/contracts_structure_checks.inc.rs",
            "src/contracts/docs/docs_static_artifact_checks.inc.rs",
            "src/contracts/docs/docs_static_index_checks.inc.rs",
            "src/contracts/docs/docs_static_metadata_checks.inc.rs",
            "src/contracts/docs/docs_static_support.inc.rs",
            "src/contracts/docs/docs_static_surface_checks.inc.rs",
            "src/contracts/make/surface_contracts.rs",
            "src/contracts/make/wrapper_contracts.rs",
            "src/contracts/ops/common/mod.rs",
            "src/contracts/root/root_static_contract_docs_checks.inc.rs",
            "src/contracts/root/root_static_hygiene_checks.inc.rs",
            "src/contracts/root/root_static_manifest_checks.inc.rs",
            "src/contracts/root/root_static_surface_checks.inc.rs",
            "src/contracts/root/root_static_workspace_checks.inc.rs",
            "src/core/checks/ops/ops/inventory_and_artifact_checks.rs",
            "src/core/ops_inventory/summary_and_fs_scan.rs",
            "src/runtime_entry.rs",
            "src/runtime_entry_checks_surface.rs",
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
            "src/commands/control_plane_docker_runtime_helpers.rs",
            "src/commands/docs/runtime/reference_page_generators.rs",
            "src/contracts/engine_model.rs",
            "src/contracts/docker/mod.rs",
            "src/commands/control_plane_contracts.rs",
            "src/runtime_entry.rs",
        ],
    );
}

#[test]
fn env_var_calls_are_constrained_to_explicit_allowlist() {
    assert_only_allowlisted_paths(
        "std::env::var(",
        &[
            "src/commands/control_plane_contracts.rs",
            "src/commands/ops/support/domain_support.rs",
            "src/contracts/docker/contracts_effect_checks.rs",
            "src/contracts/engine_model.rs",
        ],
    );
}

#[test]
fn network_client_calls_are_constrained_to_explicit_allowlist() {
    assert_only_allowlisted_paths("reqwest::", &[]);
    assert_only_allowlisted_paths("ureq::", &[]);
}
