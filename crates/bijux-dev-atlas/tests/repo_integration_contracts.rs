// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

#[path = "repo_integration_contracts/surface_checks.rs"]
mod repo_integration_contracts_surface_checks;

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
    let text = read(&root.join("ops/docker/images/runtime/Dockerfile"));
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
        "crates/bijux-atlas/src/bin/atlas-server.rs",
        "crates/bijux-atlas/src/config/mod.rs",
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
            rel.starts_with("crates/")
                .then(|| rel.to_string_lossy().to_string())
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

    let expected_version = "0.1.1";
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

        let package_features = package["features"].as_object().expect("package features");
        let package_manifest_path =
            Path::new(package["manifest_path"].as_str().expect("manifest path"));
        let package_dir = package_manifest_path.parent().expect("package dir");
        let mut local_dependencies = Vec::new();

        for dependency in package["dependencies"].as_array().expect("dependencies") {
            let Some(dep_path) = dependency["path"].as_str() else {
                continue;
            };
            let dep_name = dependency["name"]
                .as_str()
                .expect("dependency name")
                .to_string();
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

        if name == "bijux-atlas" {
            assert!(
                !local_dependencies
                    .iter()
                    .any(|dep_name| dep_name == "bijux-dev-atlas"),
                "bijux-atlas must not depend on dev-only crate bijux-dev-atlas"
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
