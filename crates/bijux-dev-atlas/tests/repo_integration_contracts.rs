// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

fn collect_files(root: &Path, extension: &str, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files(&path, extension, out);
            } else if path.extension().and_then(|value| value.to_str()) == Some(extension) {
                out.push(path);
            }
        }
    }
}

fn markdown_files_under(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files(root, "md", &mut files);
    files.sort();
    files
}

fn load_json(path: &Path) -> Value {
    serde_json::from_str(&read(path))
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()))
}

fn collect_prefixed_env_tokens(text: &str) -> BTreeSet<String> {
    text.split(|ch: char| !(ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_'))
        .filter(|token| {
            (token.starts_with("ATLAS_") && token.len() > "ATLAS_".len())
                || (token.starts_with("BIJUX_") && token.len() > "BIJUX_".len())
        })
        .map(str::to_string)
        .collect()
}

fn render_chart_env_keys(root: &Path) -> BTreeSet<String> {
    let output = Command::new("helm")
        .current_dir(root)
        .args([
            "template",
            "atlas-default",
            "ops/k8s/charts/bijux-atlas",
            "-f",
            "ops/k8s/charts/bijux-atlas/values.yaml",
        ])
        .output()
        .expect("helm template");
    assert!(
        output.status.success(),
        "helm template must succeed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered = String::from_utf8(output.stdout).expect("helm template utf8");
    let mut env_names = BTreeSet::new();
    for line in rendered.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_suffix(':') {
            if name.starts_with("ATLAS_") || name.starts_with("BIJUX_") {
                env_names.insert(name.to_string());
            }
        }
        if let Some(name) = trimmed.strip_prefix("- name: ") {
            if name.starts_with("ATLAS_") || name.starts_with("BIJUX_") {
                env_names.insert(name.to_string());
            }
        }
    }
    env_names
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

fn extract_shell_recipe_commands(path: &Path) -> BTreeMap<String, Vec<String>> {
    let mut out = BTreeMap::<String, Vec<String>>::new();
    let text = read(path);
    let mut current = None::<String>;
    for line in text.lines() {
        if line.starts_with('\t') {
            if let Some(target) = current.as_ref() {
                out.entry(target.clone())
                    .or_default()
                    .push(line.trim().to_string());
            }
            continue;
        }
        current = None;
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with('.')
            || trimmed.contains(":=")
            || trimmed.contains("?=")
        {
            continue;
        }
        let Some((head, _)) = trimmed.split_once(':') else {
            continue;
        };
        let name = head.trim();
        if name
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
        {
            out.entry(name.to_string()).or_default();
            current = Some(name.to_string());
        }
    }
    out
}

fn parse_make_registry() -> Vec<(String, Vec<String>, String)> {
    let root = repo_root();
    let json = load_json(&root.join("configs/ops/make-target-registry.json"));
    json["targets"]
        .as_array()
        .expect("targets array")
        .iter()
        .filter_map(|row| {
            Some((
                row.get("name")?.as_str()?.to_string(),
                row.get("defined_in")?
                    .as_array()?
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>(),
                row.get("visibility")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
                    .to_string(),
            ))
        })
        .collect()
}

#[test]
fn dockerfile_copy_sources_exist_and_stay_within_root_authority() {
    let root = repo_root();
    let text = read(&root.join("docker/images/runtime/Dockerfile"));
    let mut copy_sources = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("COPY ") {
            continue;
        }
        let rest = trimmed.trim_start_matches("COPY ").trim();
        if rest.starts_with("--from=") {
            continue;
        }
        let parts = rest.split_whitespace().collect::<Vec<_>>();
        if parts.len() < 2 {
            continue;
        }
        for source in &parts[..parts.len() - 1] {
            if source.starts_with("--") || source.starts_with('$') {
                continue;
            }
            let normalized = source.trim_matches('"').trim_start_matches("./");
            copy_sources.push(normalized.to_string());
            assert!(
                !normalized.starts_with("ops/")
                    && !normalized.starts_with("docs/")
                    && !normalized.starts_with("make/"),
                "runtime Dockerfile COPY must use canonical root/configs/crates inputs only: {normalized}"
            );
            assert!(
                root.join(normalized).exists(),
                "runtime Dockerfile COPY source is missing: {normalized}"
            );
        }
    }
    assert!(
        !copy_sources.is_empty(),
        "runtime Dockerfile must declare at least one COPY source"
    );
    assert!(
        copy_sources.iter().any(|source| source == "Cargo.toml"),
        "runtime Dockerfile must COPY the root Cargo.toml"
    );
    assert!(
        copy_sources
            .iter()
            .any(|source| source.starts_with("configs/")),
        "runtime Dockerfile must COPY canonical configs inputs"
    );
    assert!(
        copy_sources.iter().any(|source| source == "crates"),
        "runtime Dockerfile must COPY the crates workspace"
    );
}

#[test]
fn root_symlinks_and_dockerignore_follow_surface_contract() {
    let root = repo_root();
    let manifest = load_json(&root.join("ops/inventory/root-surface.json"));
    let entries = manifest["entries"].as_object().expect("entries object");

    for entry in fs::read_dir(&root).expect("repo root").flatten() {
        let path = entry.path();
        if !path.is_symlink() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let kind = entries
            .get(&name)
            .and_then(|value| value.get("kind"))
            .and_then(Value::as_str);
        assert_eq!(
            kind,
            Some("symlink"),
            "root symlink must be declared as a symlink in ops/inventory/root-surface.json: {name}"
        );
    }

    let dockerignore = read(&root.join(".dockerignore"));
    for required in [
        ".git",
        ".github",
        "artifacts",
        "ops/_generated",
        "**/target",
    ] {
        assert!(
            dockerignore.lines().any(|line| line.trim() == required),
            ".dockerignore must include `{required}`"
        );
    }
}

#[test]
fn runtime_config_docs_and_helm_env_surface_match_declared_config_keys() {
    let root = repo_root();
    let config_doc = read(&root.join("docs/operations/config.md"));
    for required in [
        "crates/bijux-atlas-server/docs/generated/runtime-startup-config.schema.json",
        "crates/bijux-atlas-server/docs/generated/runtime-startup-config.md",
    ] {
        assert!(
            config_doc.contains(required),
            "runtime config doc must reference `{required}`"
        );
        assert!(
            root.join(required).is_file(),
            "runtime config doc reference must exist: {required}"
        );
    }

    let config_keys = load_json(&root.join("docs/reference/contracts/schemas/CONFIG_KEYS.json"));
    let declared = config_keys["env_keys"]
        .as_array()
        .expect("env_keys array")
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<BTreeSet<_>>();

    let env_names = render_chart_env_keys(&root);
    for env_name in &env_names {
        assert!(
            declared.contains(env_name),
            "helm deployment env var must exist in docs/reference/contracts/schemas/CONFIG_KEYS.json: {env_name}"
        );
    }

    let config_schema = load_json(&root.join("configs/contracts/env.schema.json"));
    let schema_keys = config_schema["allowed_env"]
        .as_array()
        .expect("allowed_env array")
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    for env_name in env_names {
        assert!(
            schema_keys.contains(&env_name),
            "helm deployment env var must exist in configs/contracts/env.schema.json: {env_name}"
        );
    }
}

#[test]
fn server_runtime_prefixed_env_reads_stay_inside_the_env_contract() {
    let root = repo_root();
    let config_schema = load_json(&root.join("configs/contracts/env.schema.json"));
    let schema_keys = config_schema["allowed_env"]
        .as_array()
        .expect("allowed_env array")
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<BTreeSet<_>>();

    let mut runtime_keys = BTreeSet::new();
    for relative_path in [
        "crates/bijux-atlas-server/src/main.rs",
        "crates/bijux-atlas-server/src/config/mod.rs",
    ] {
        let text = read(&root.join(relative_path));
        runtime_keys.extend(collect_prefixed_env_tokens(&text));
    }

    let missing = runtime_keys
        .difference(&schema_keys)
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "runtime-prefixed env keys must be declared in configs/contracts/env.schema.json:\n{}",
        missing.join("\n")
    );
}

#[test]
fn workspace_members_exactly_match_the_crate_directories_on_disk() {
    let root = repo_root();
    let output = Command::new("cargo")
        .current_dir(&root)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .expect("cargo metadata");
    assert!(
        output.status.success(),
        "cargo metadata must succeed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: Value = serde_json::from_slice(&output.stdout).expect("cargo metadata json");

    let declared = metadata["packages"]
        .as_array()
        .expect("metadata packages")
        .iter()
        .filter_map(|package| package["manifest_path"].as_str())
        .filter_map(|manifest_path| {
            let path = Path::new(manifest_path);
            let parent = path.parent()?;
            let rel = parent.strip_prefix(&root).ok()?;
            rel.starts_with("crates/").then(|| rel.to_string_lossy().to_string())
        })
        .collect::<BTreeSet<_>>();

    let actual = fs::read_dir(root.join("crates"))
        .expect("read crates dir")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter_map(|path| {
            path.strip_prefix(&root)
                .ok()
                .map(|rel| rel.to_string_lossy().to_string())
        })
        .collect::<BTreeSet<_>>();

    assert_eq!(
        declared, actual,
        "workspace members must match the crate directories on disk"
    );
}

#[test]
fn workspace_package_metadata_stays_acyclic_and_consistent() {
    let root = repo_root();
    let output = Command::new("cargo")
        .current_dir(&root)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .expect("cargo metadata");
    assert!(
        output.status.success(),
        "cargo metadata must succeed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: Value = serde_json::from_slice(&output.stdout).expect("cargo metadata json");
    let packages = metadata["packages"].as_array().expect("metadata packages");

    let mut packages_by_name = BTreeMap::new();
    for package in packages {
        let name = package["name"].as_str().expect("package name").to_string();
        let previous = packages_by_name.insert(name.clone(), package);
        assert!(previous.is_none(), "duplicate crate name detected: {name}");
    }

    let expected_version = "0.1.0";
    let expected_rust_version = "1.84.1";
    let expected_license = "Apache-2.0";
    let mut adjacency = BTreeMap::<String, Vec<String>>::new();

    for (name, package) in &packages_by_name {
        assert_eq!(
            package["version"].as_str(),
            Some(expected_version),
            "workspace crate version drift for {name}"
        );
        assert_eq!(
            package["rust_version"].as_str(),
            Some(expected_rust_version),
            "workspace crate rust-version drift for {name}"
        );
        assert_eq!(
            package["license"].as_str(),
            Some(expected_license),
            "workspace crate license drift for {name}"
        );

        let package_features = package["features"]
            .as_object()
            .expect("package features");
        let package_manifest_path = Path::new(package["manifest_path"].as_str().expect("manifest path"));
        let package_dir = package_manifest_path.parent().expect("package dir");
        let mut local_dependencies = Vec::new();

        for dependency in package["dependencies"].as_array().expect("dependencies") {
            let Some(dep_path) = dependency["path"].as_str() else {
                continue;
            };
            let dep_name = dependency["name"].as_str().expect("dependency name").to_string();
            let dep_path = Path::new(dep_path);
            assert!(
                dep_path.starts_with(root.join("crates")),
                "path dependency for {name} must point into the workspace crates directory: {dep_name}"
            );
            assert!(
                packages_by_name.contains_key(&dep_name),
                "path dependency for {name} references missing workspace crate: {dep_name}"
            );
            assert!(
                dep_path.join("Cargo.toml").is_file(),
                "path dependency for {name} must point to a crate directory with Cargo.toml: {dep_name}"
            );
            assert_ne!(
                dep_path, package_dir,
                "crate {name} must not depend on itself"
            );

            if let Some(requested_features) = dependency["features"].as_array() {
                let dependency_package = packages_by_name
                    .get(&dep_name)
                    .expect("dependency package present");
                let dependency_features = dependency_package["features"]
                    .as_object()
                    .expect("dependency features");
                for feature in requested_features.iter().filter_map(Value::as_str) {
                    assert!(
                        feature == "default"
                            || dependency_features.contains_key(feature)
                            || package_features.contains_key(feature),
                        "crate {name} requests missing feature `{feature}` from dependency {dep_name}"
                    );
                }
            }

            local_dependencies.push(dep_name);
        }

        if name == "bijux-atlas-server" {
            assert!(
                !local_dependencies
                    .iter()
                    .any(|dep_name| dep_name == "bijux-dev-atlas"),
                "bijux-atlas-server must not depend on dev-only crate bijux-dev-atlas"
            );
        }
        adjacency.insert(name.clone(), local_dependencies);
    }

    fn visit(
        node: &str,
        adjacency: &BTreeMap<String, Vec<String>>,
        visiting: &mut BTreeSet<String>,
        visited: &mut BTreeSet<String>,
    ) {
        if visited.contains(node) {
            return;
        }
        assert!(
            visiting.insert(node.to_string()),
            "cyclic workspace dependency detected at {node}"
        );
        if let Some(children) = adjacency.get(node) {
            for child in children {
                visit(child, adjacency, visiting, visited);
            }
        }
        visiting.remove(node);
        visited.insert(node.to_string());
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for name in adjacency.keys() {
        visit(name, &adjacency, &mut visiting, &mut visited);
    }
}

#[test]
fn docs_workflows_copy_the_built_site_from_mkdocs_site_dir() {
    let root = repo_root();
    let mkdocs = read(&root.join("mkdocs.yml"));
    assert!(
        mkdocs.contains("site_dir: artifacts/docs/site"),
        "mkdocs.yml must keep the authoritative docs site_dir"
    );

    for workflow in [
        ".github/workflows/docs-audit.yml",
        ".github/workflows/docs-only.yml",
        ".github/workflows/ci-pr.yml",
    ] {
        let text = read(&root.join(workflow));
        assert!(
            text.contains("cp -R artifacts/docs/site \"artifacts/${RUN_ID}/site-preview\""),
            "{workflow} must copy the built docs preview from mkdocs site_dir"
        );
        assert!(
            !text.contains("cp -R site \"artifacts/${RUN_ID}/site-preview\""),
            "{workflow} must not rely on the obsolete default MkDocs output directory"
        );
    }
}

#[test]
fn ci_workflows_keep_dependency_inputs_and_action_refs_deterministic() {
    let root = repo_root();
    let workflows_dir = root.join(".github/workflows");
    let mut workflow_files = fs::read_dir(&workflows_dir)
        .expect("workflows dir")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            matches!(
                path.extension().and_then(|value| value.to_str()),
                Some("yml" | "yaml")
            )
        })
        .collect::<Vec<_>>();
    workflow_files.sort();

    let sha_ref_len = 40;
    let mut cargo_without_locked = Vec::<String>::new();
    let mut uses_not_pinned = Vec::<String>::new();
    let mut cargo_update_calls = Vec::<String>::new();
    let mut artifact_paths_outside_root = Vec::<String>::new();
    let mut default_cluster_assumptions = Vec::<String>::new();

    for workflow in workflow_files {
        let text = read(&workflow);
        let rel = workflow
            .strip_prefix(&root)
            .expect("workflow relative path")
            .display()
            .to_string();

        for (line_index, raw_line) in text.lines().enumerate() {
            let line = raw_line.trim();
            let line_number = line_index + 1;
            if line.contains("cargo update") {
                cargo_update_calls.push(format!("{rel}:{line_number}"));
            }
            if line.contains("kubectl config use-context")
                || line.contains("kubectl config current-context")
                || line.contains("default cluster")
            {
                default_cluster_assumptions.push(format!("{rel}:{line_number}: {line}"));
            }
            if let Some(action_ref) = line.strip_prefix("uses: ") {
                let Some((_, pinned_ref)) = action_ref.rsplit_once('@') else {
                    uses_not_pinned.push(format!("{rel}:{line_number}"));
                    continue;
                };
                if pinned_ref.len() != sha_ref_len
                    || !pinned_ref.chars().all(|ch| ch.is_ascii_hexdigit())
                {
                    uses_not_pinned.push(format!("{rel}:{line_number}"));
                }
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
fn mkdocs_config_enables_redirects_plugin_for_legacy_markdown_paths() {
    let root = repo_root();
    let mkdocs = read(&root.join("mkdocs.yml"));
    assert!(
        mkdocs.contains("- redirects:"),
        "mkdocs.yml must enable the redirects plugin"
    );
    assert!(
        mkdocs.contains("redirect_maps:"),
        "mkdocs.yml must declare redirect_maps for legacy markdown paths"
    );
    assert!(
        mkdocs.contains("_generated/topic-index.md: _internal/generated/topic-index.md"),
        "mkdocs.yml must redirect legacy _generated topic index pages"
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
fn quickstart_command_is_backed_by_cli_help() {
    let root = repo_root();
    let start_here = read(&root.join("docs/start-here.md"));
    let command = start_here
        .lines()
        .find(|line| {
            line.trim_start()
                .starts_with("bijux dev atlas demo quickstart")
        })
        .map(str::trim)
        .expect("docs/start-here.md quickstart command");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["demo", "quickstart", "--help"])
        .output()
        .expect("demo quickstart help");
    assert!(output.status.success(), "demo quickstart help must succeed");
    let help = String::from_utf8(output.stdout).expect("utf8");
    for required in ["bijux-dev-atlas demo quickstart", "--format", "--out"] {
        assert!(
            help.contains(required),
            "demo quickstart help must contain `{required}`"
        );
    }
    assert!(
        command == "bijux dev atlas demo quickstart --format json",
        "docs/start-here.md must keep the canonical quickstart command"
    );
}

#[test]
fn ci_wrappers_and_workflows_use_the_same_named_suites() {
    let root = repo_root();
    let ci_make = read(&root.join("make/ci.mk"));
    let ci_workflow = read(&root.join(".github/workflows/ci-pr.yml"));
    for suite in ["ci_pr", "ci_fast"] {
        assert!(
            ci_make.contains(&format!("--suite {suite}")),
            "make/ci.mk must reference suite `{suite}`"
        );
        assert!(
            ci_workflow.contains(suite),
            "ci-pr workflow must reference suite `{suite}`"
        );
    }
}

#[test]
fn public_make_targets_map_to_one_control_plane_entry_and_stay_thin() {
    let root = repo_root();
    let public_targets = load_json(&root.join("configs/make/public-targets.json"))
        ["public_targets"]
        .as_array()
        .expect("public_targets array")
        .iter()
        .filter_map(|value| value.get("name").and_then(Value::as_str))
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let registry = parse_make_registry();
    let registry_map = registry
        .into_iter()
        .map(|(name, defined_in, visibility)| (name, (defined_in, visibility)))
        .collect::<BTreeMap<_, _>>();

    let mut recipes_by_target = BTreeMap::<String, Vec<String>>::new();
    let makefiles = fs::read_dir(root.join("make"))
        .expect("make")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("mk"))
        .collect::<Vec<_>>();
    for path in makefiles {
        for (target, recipes) in extract_shell_recipe_commands(&path) {
            recipes_by_target.entry(target).or_default().extend(recipes);
        }
    }

    for target in public_targets {
        let (defined_in, visibility) = registry_map
            .get(&target)
            .unwrap_or_else(|| panic!("public target missing from make target registry: {target}"));
        assert_eq!(
            visibility, "public",
            "public target must be public in registry: {target}"
        );
        assert_eq!(
            defined_in.len(),
            1,
            "public target must map to exactly one defining makefile: {target}"
        );

        let recipes = recipes_by_target
            .get(&target)
            .unwrap_or_else(|| panic!("missing recipe for public target: {target}"));

        let substantive = recipes
            .iter()
            .filter(|line| {
                let trimmed = line.trim_start_matches('@').trim();
                !trimmed.starts_with("printf ")
                    && !trimmed.starts_with("mkdir -p ")
                    && !trimmed.starts_with("rm -f ")
                    && !trimmed.starts_with("cp ")
                    && !trimmed.starts_with("cat ")
                    && !trimmed.starts_with("tee ")
            })
            .collect::<Vec<_>>();

        if substantive.is_empty() {
            continue;
        }

        let delegated = substantive
            .iter()
            .filter(|line| {
                line.contains("$(DEV_ATLAS)") || line.contains("cargo ") || line.contains("$(MAKE)")
            })
            .count();
        if ["help", "make-target-list"].contains(&target.as_str()) {
            continue;
        }
        assert!(
            delegated >= 1,
            "public target must delegate through DEV_ATLAS, cargo, or make: {target}"
        );
    }
}

#[test]
fn docs_and_automation_surfaces_do_not_reference_removed_control_plane_paths() {
    let root = repo_root();
    let mut files = markdown_files_under(&root.join("docs"));
    files.extend(markdown_files_under(&root.join("make")));
    for path in [
        root.join("README.md"),
        root.join("CONTRIBUTING.md"),
        root.join("SECURITY.md"),
        root.join("CHANGELOG.md"),
    ] {
        files.push(path);
    }
    let forbidden = ["atlasctl", "scripts/", "xtask", "tools/"];
    let mut violations = Vec::new();
    for file in files {
        let content = read(&file);
        let rel = file
            .strip_prefix(&root)
            .expect("repo relative")
            .display()
            .to_string();
        for needle in forbidden {
            if content.contains(needle) {
                violations.push(format!("{rel} contains `{needle}`"));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "docs and automation surfaces must not reference removed control-plane paths:\n{}",
        violations.join("\n")
    );
}

#[test]
fn root_dockerfile_symlink_points_to_canonical_runtime_dockerfile() {
    let root = repo_root();
    let dockerfile = root.join("Dockerfile");
    assert!(
        dockerfile.is_symlink(),
        "root Dockerfile must remain a symlink shim"
    );
    let target = fs::read_link(&dockerfile).expect("Dockerfile symlink target");
    assert_eq!(
        target,
        PathBuf::from("docker/images/runtime/Dockerfile"),
        "root Dockerfile must point to the canonical runtime Dockerfile"
    );
    assert!(
        root.join(&target).is_file(),
        "canonical runtime Dockerfile target must exist"
    );

    let allowlist = load_json(&root.join("configs/repo/symlink-allowlist.json"));
    let declared = allowlist["root"]["Dockerfile"]
        .as_str()
        .expect("root Dockerfile symlink allowlist entry");
    assert_eq!(
        declared, "docker/images/runtime/Dockerfile",
        "symlink allowlist must match the canonical runtime Dockerfile target"
    );
}

#[test]
fn contracts_makefile_is_the_only_public_contract_gate_entrypoint() {
    let root = repo_root();
    let registry = parse_make_registry();
    let docs = read(&root.join("docs/_internal/generated/make-targets.md"));

    for (name, defined_in, visibility) in registry {
        if !name.starts_with("contracts") || visibility != "public" {
            continue;
        }
        assert_eq!(
            defined_in,
            vec!["make/contracts.mk".to_string()],
            "public contract gate target must be defined only in make/contracts.mk: {name}"
        );
        assert!(
            docs.contains(&format!("| `{name}` | `public` | `make/contracts.mk` |")),
            "generated make target reference must document the public contract gate target: {name}"
        );
    }

    let public_mk = read(&root.join("make/public.mk"));
    assert!(
        public_mk
            .lines()
            .any(|line| line.trim() == "include make/contracts.mk"),
        "make/public.mk must delegate contract gates through make/contracts.mk"
    );
}

#[test]
#[ignore = "legacy quality-wall contract pending rewrite"]
fn config_contract_surfaces_are_versioned_referenced_and_deterministically_formatted() {
    let root = repo_root();
    let deterministic = [
        root.join("docs/reference/contracts/schemas/CONFIG_KEYS.json"),
        root.join("configs/contracts/env.schema.json"),
        root.join("configs/repo/symlink-allowlist.json"),
    ];
    for path in deterministic {
        let parsed = load_json(&path);
        assert!(
            parsed.get("schema_version").is_some(),
            "governed config contract must declare a schema version: {}",
            path.display()
        );
        assert_pretty_json_file(&path);
    }
    let make_registry = load_json(&root.join("configs/ops/make-target-registry.json"));
    assert!(
        make_registry.get("schema_version").is_some(),
        "make target registry must declare a schema version"
    );

    let config_versioning = read(&root.join("docs/development/config-versioning.md"));
    let registry_versioning =
        read(&root.join("docs/reference/registry/config-schema-versioning.md"));
    for required in [
        "docs/reference/contracts/schemas/CONFIG_KEYS.json",
        "configs/contracts/env.schema.json",
        "schema_version",
    ] {
        assert!(
            config_versioning.contains(required) || registry_versioning.contains(required),
            "config schema versioning docs must reference `{required}`"
        );
    }
}

#[test]
fn config_security_and_docker_contracts_stay_explicit_and_reviewable() {
    let root = repo_root();
    let allowlist = read(&root.join("configs/security/audit-allowlist.toml"));
    let allowlist_lines = allowlist
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect::<Vec<_>>();
    assert!(
        allowlist_lines.is_empty(),
        "security audit allowlist must stay empty unless a reviewed exception is added"
    );

    let dockerfile = read(&root.join("docker/images/runtime/Dockerfile"));
    assert!(
        !dockerfile
            .lines()
            .any(|line| line.trim_start().starts_with("ADD ")),
        "runtime Dockerfile must not use ADD"
    );
    let lowered = dockerfile.to_ascii_lowercase();
    for forbidden in ["curl | sh", "curl|sh", "wget | sh", "wget|sh"] {
        assert!(
            !lowered.contains(forbidden),
            "runtime Dockerfile must not contain `{forbidden}`"
        );
    }
    for line in dockerfile.lines().map(str::trim) {
        if let Some(image) = line.strip_prefix("FROM ") {
            let image = image.split_whitespace().next().expect("FROM image");
            assert!(
                image.contains("@sha256:"),
                "runtime Dockerfile base image must be pinned by digest: {image}"
            );
            assert!(
                !image.ends_with(":latest"),
                "runtime Dockerfile base image must not use latest: {image}"
            );
        }
    }
}

#[test]
#[ignore = "legacy quality-wall contract pending rewrite"]
fn quality_wall_doc_ties_required_contracts_lanes_and_repo_surfaces_together() {
    let root = repo_root();
    let quality_wall = read(&root.join("docs/operations/release/quality-wall.md"));
    for required in [
        "ops/policy/required-contracts.json",
        "ops/_generated.example/contracts-required.json",
        "make/contracts.mk",
        "docker/images/runtime/Dockerfile",
        "docs/_internal/generated/make-targets.md",
        "configs/contracts/env.schema.json",
        "docs/reference/contracts/schemas/CONFIG_KEYS.json",
        "local",
        "pr",
        "merge",
        "release",
    ] {
        assert!(
            quality_wall.contains(required),
            "quality wall doc must reference `{required}`"
        );
    }
}
