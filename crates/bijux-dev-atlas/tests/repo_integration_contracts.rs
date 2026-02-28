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
    fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
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
        text, expected,
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
        copy_sources.iter().any(|source| source.starts_with("configs/")),
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
    let manifest = load_json(&root.join("root-surface.json"));
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
            "root symlink must be declared as a symlink in root-surface.json: {name}"
        );
    }

    let dockerignore = read(&root.join(".dockerignore"));
    for required in [".git", ".github", "artifacts", "ops/_generated", "**/target"] {
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

    let deployment = read(&root.join("ops/k8s/charts/bijux-atlas/templates/deployment.yaml"));
    let mut env_names = BTreeSet::new();
    for line in deployment.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix("- name: ") {
            let name = name.trim();
            if name.starts_with("ATLAS_") || name.starts_with("BIJUX_") {
                env_names.insert(name.to_string());
            }
        }
    }
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
fn quickstart_command_is_backed_by_cli_help() {
    let root = repo_root();
    let start_here = read(&root.join("docs/START_HERE.md"));
    let command = start_here
        .lines()
        .find(|line| line.trim_start().starts_with("bijux dev atlas demo quickstart"))
        .map(str::trim)
        .expect("docs/START_HERE.md quickstart command");
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
        "docs/START_HERE.md must keep the canonical quickstart command"
    );
}

#[test]
fn ci_wrappers_and_workflows_use_the_same_named_suites() {
    let root = repo_root();
    let ci_make = read(&root.join("make/makefiles/ci.mk"));
    let ci_workflow = read(&root.join(".github/workflows/ci-pr.yml"));
    for suite in ["ci_pr", "ci_fast"] {
        assert!(
            ci_make.contains(&format!("--suite {suite}")),
            "make/makefiles/ci.mk must reference suite `{suite}`"
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
    let public_targets = load_json(&root.join("configs/make/public-targets.json"))["public_targets"]
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
    let mut makefiles = fs::read_dir(root.join("make/makefiles"))
        .expect("make/makefiles")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("mk"))
        .collect::<Vec<_>>();
    makefiles.push(root.join("make/contracts.mk"));
    makefiles.push(root.join("make/public.mk"));
    for path in makefiles {
        for (target, recipes) in extract_shell_recipe_commands(&path) {
            recipes_by_target.entry(target).or_default().extend(recipes);
        }
    }

    for target in public_targets {
        let (defined_in, visibility) = registry_map
            .get(&target)
            .unwrap_or_else(|| panic!("public target missing from make target registry: {target}"));
        assert_eq!(visibility, "public", "public target must be public in registry: {target}");
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
    assert!(dockerfile.is_symlink(), "root Dockerfile must remain a symlink shim");
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
        declared,
        "docker/images/runtime/Dockerfile",
        "symlink allowlist must match the canonical runtime Dockerfile target"
    );
}

#[test]
fn contracts_makefile_is_the_only_public_contract_gate_entrypoint() {
    let root = repo_root();
    let registry = parse_make_registry();
    let docs = read(&root.join("docs/_generated/make-targets.md"));

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
        public_mk.lines().any(|line| line.trim() == "include make/contracts.mk"),
        "make/public.mk must delegate contract gates through make/contracts.mk"
    );
}

#[test]
fn config_contract_surfaces_are_versioned_referenced_and_deterministically_formatted() {
    let root = repo_root();
    let deterministic = [
        root.join("docs/reference/contracts/schemas/CONFIG_KEYS.json"),
        root.join("configs/contracts/env.schema.json"),
        root.join("configs/repo/symlink-allowlist.json"),
    ];
    for path in deterministic {
        let parsed = load_json(&path);
        assert!(parsed.get("schema_version").is_some(), "governed config contract must declare a schema version: {}", path.display());
        assert_pretty_json_file(&path);
    }
    let make_registry = load_json(&root.join("configs/ops/make-target-registry.json"));
    assert!(
        make_registry.get("schema_version").is_some(),
        "make target registry must declare a schema version"
    );

    let config_versioning = read(&root.join("docs/development/config-versioning.md"));
    let registry_versioning = read(&root.join("docs/reference/registry/config-schema-versioning.md"));
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
        !dockerfile.lines().any(|line| line.trim_start().starts_with("ADD ")),
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
fn quality_wall_doc_ties_required_contracts_lanes_and_repo_surfaces_together() {
    let root = repo_root();
    let quality_wall = read(&root.join("docs/operations/release/quality-wall.md"));
    for required in [
        "ops/policy/required-contracts.json",
        "artifacts/contracts/required.json",
        "make/contracts.mk",
        "docker/images/runtime/Dockerfile",
        "docs/_generated/make-targets.md",
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
