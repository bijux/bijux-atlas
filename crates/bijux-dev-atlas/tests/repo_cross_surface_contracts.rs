// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn assert_pretty_json_file(path: &Path) {
    let text = read(path);
    let parsed: Value = serde_json::from_str(&text)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()));
    let expected = format!(
        "{}\n",
        serde_json::to_string_pretty(&parsed)
            .unwrap_or_else(|err| panic!("failed to render {}: {err}", path.display()))
    );
    assert_eq!(
        text,
        expected,
        "governed json file must use deterministic pretty formatting: {}",
        path.display()
    );
}

#[test]
fn ci_workflows_keep_dependency_inputs_and_action_refs_deterministic() {
    let root = repo_root();
    let workflows_root = root.join(".github/workflows");
    let mut workflow_paths = fs::read_dir(&workflows_root)
        .expect("workflow directory")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            matches!(
                path.extension().and_then(|value| value.to_str()),
                Some("yml" | "yaml")
            )
        })
        .collect::<Vec<_>>();
    workflow_paths.sort();

    let mut cargo_update_calls = Vec::new();
    let mut cargo_without_locked = Vec::new();
    let mut uses_not_pinned = Vec::new();
    let mut artifact_paths_outside_root = Vec::new();
    let mut default_cluster_assumptions = Vec::new();

    for path in workflow_paths {
        let rel = path
            .strip_prefix(&root)
            .expect("repo relative workflow path")
            .display()
            .to_string();
        let text = read(&path);
        for (index, raw_line) in text.lines().enumerate() {
            let line_number = index + 1;
            let line = raw_line.trim();

            if line.starts_with("uses:")
                && line.contains('@')
                && !line.starts_with("uses: docker://")
            {
                let revision = line.split('@').nth(1).map(str::trim).unwrap_or_default();
                if revision.len() != 40 || !revision.chars().all(|ch| ch.is_ascii_hexdigit()) {
                    uses_not_pinned.push(format!("{rel}:{line_number}: {line}"));
                }
            }

            if line.contains("cargo update") {
                cargo_update_calls.push(format!("{rel}:{line_number}: {line}"));
            }

            if line.contains("kubectl ")
                && !line.contains("--context ")
                && !line.contains("config current-context")
            {
                default_cluster_assumptions.push(format!("{rel}:{line_number}: {line}"));
            }

            if let Some(path_value) = line.strip_prefix("path: ") {
                let normalized = path_value.trim_matches('"').trim();
                if normalized != "|"
                    && !normalized.starts_with("artifacts/")
                    && !normalized.starts_with(".cache/")
                {
                    artifact_paths_outside_root.push(format!("{rel}:{line_number}: {normalized}"));
                }
            }

            if line.starts_with("- name:")
                || line.starts_with("uses:")
                || line.starts_with("printf ")
                || !line.contains("cargo ")
            {
                continue;
            }
            let cargo_index = line.find("cargo ").expect("cargo command index");
            let cargo_command = &line[cargo_index..];
            if cargo_command.starts_with("cargo install --locked")
                || cargo_command.starts_with("cargo deny ")
                || cargo_command.starts_with("cargo audit")
            {
                continue;
            }
            let allows_unlock = cargo_command.starts_with("cargo generate-lockfile");
            if !allows_unlock && !cargo_command.contains("--locked") {
                cargo_without_locked.push(format!("{rel}:{line_number}: {cargo_command}"));
            }
        }

        if rel == ".github/workflows/docs-audit.yml" || rel == ".github/workflows/docs-only.yml" {
            assert!(
                text.contains("python3 -m pip install -r configs/docs/requirements.lock.txt"),
                "{rel} must install Python docs dependencies from requirements.lock.txt"
            );
            assert!(
                text.contains("npm ci --prefix configs/docs"),
                "{rel} must install Node docs dependencies with npm ci"
            );
        }
    }

    assert!(
        cargo_update_calls.is_empty(),
        "workflows must not run cargo update:\n{}",
        cargo_update_calls.join("\n")
    );
    assert!(
        cargo_without_locked.is_empty(),
        "workflow cargo commands must use --locked (except cargo generate-lockfile):\n{}",
        cargo_without_locked.join("\n")
    );
    assert!(
        uses_not_pinned.is_empty(),
        "workflow action refs must be pinned to full commit SHAs:\n{}",
        uses_not_pinned.join("\n")
    );
    assert!(
        artifact_paths_outside_root.is_empty(),
        "workflow artifact paths must stay inside the canonical artifacts/ root:\n{}",
        artifact_paths_outside_root.join("\n")
    );
    assert!(
        default_cluster_assumptions.is_empty(),
        "workflows must not assume an ambient default Kubernetes context:\n{}",
        default_cluster_assumptions.join("\n")
    );
}

#[test]
fn governed_json_configuration_surfaces_stay_pretty_printed() {
    let root = repo_root();
    for relative_path in [
        "configs/contracts/env.schema.json",
        "ops/k8s/charts/bijux-atlas/values.schema.json",
        "configs/rust/toolchain.json",
        "ops/inventory/toolchain.json",
    ] {
        assert_pretty_json_file(&root.join(relative_path));
    }
}

#[test]
fn helm_values_do_not_expose_dead_runtime_tuning_branches() {
    let root = repo_root();
    let chart_values = read(&root.join("ops/k8s/charts/bijux-atlas/values.yaml"));
    let chart_schema = read(&root.join("ops/k8s/charts/bijux-atlas/values.schema.json"));
    let perf_profile = read(&root.join("ops/k8s/values/perf.yaml"));

    for forbidden in ["\nrateLimits:\n", "\nconcurrency:\n"] {
        assert!(
            !chart_values.contains(forbidden),
            "chart values must not expose dead runtime tuning branch `{forbidden}`"
        );
    }
    for forbidden in ["\"rateLimits\"", "\"concurrency\""] {
        assert!(
            !chart_schema.contains(forbidden),
            "chart values schema must not expose dead runtime tuning branch {forbidden}"
        );
    }
    assert!(
        !perf_profile.contains("\nconcurrency:\n"),
        "perf profile must not override dead runtime tuning branches"
    );
}

#[test]
fn root_contract_documents_the_cross_surface_invariants() {
    let root = repo_root();
    let contract = read(&root.join("governance/README.md"));

    for invariant in [
        "Invariant: Helm env emitted by the chart must stay a subset of the runtime allowlist declared in `configs/contracts/env.schema.json`.",
        "Invariant: Every rollout profile under `ops/k8s/values/` must render successfully.",
        "Invariant: Every crate directory under `crates/` must be declared as a workspace member in the root `Cargo.toml`.",
        "Invariant: `mkdocs build --strict` must publish into the configured `site_dir`.",
        "Invariant: Docs must not contain references to missing pages.",
        "Invariant: No rollout profile may violate Helm chart fail guards.",
        "Invariant: Policy-surface configuration files must not be committed in minified form.",
        "Invariant: Unknown runtime `ATLAS_*` or `BIJUX_*` environment variables must fail startup unless the explicit local-dev override is enabled.",
        "Invariant: No single runtime behavior may be controlled through duplicate semantic environment variable names.",
    ] {
        assert!(
            contract.contains(invariant),
            "governance/README.md must document cross-surface invariant `{invariant}`"
        );
    }
}
