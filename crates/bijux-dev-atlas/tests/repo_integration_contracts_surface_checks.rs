// SPDX-License-Identifier: Apache-2.0

use super::*;

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
