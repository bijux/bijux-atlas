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
    fn contains_forbidden_path_segment(content: &str, segment: &str) -> bool {
        let mut search_from = 0usize;
        while let Some(index) = content[search_from..].find(segment) {
            let absolute = search_from + index;
            let boundary_ok = absolute == 0
                || !content[..absolute]
                    .chars()
                    .last()
                    .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_');
            if boundary_ok {
                return true;
            }
            search_from = absolute + segment.len();
        }
        false
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
            let found = match needle {
                "scripts/" | "tools/" => contains_forbidden_path_segment(&content, needle),
                _ => content.contains(needle),
            };
            if found {
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
        PathBuf::from("ops/docker/images/runtime/Dockerfile"),
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
        declared, "ops/docker/images/runtime/Dockerfile",
        "symlink allowlist must match the canonical runtime Dockerfile target"
    );
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

    let dockerfile = read(&root.join("ops/docker/images/runtime/Dockerfile"));
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
