// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[derive(Debug, Deserialize)]
struct MakeTargetRegistry {
    schema_version: u64,
    targets: Vec<MakeTargetEntry>,
}

#[derive(Debug, Deserialize)]
struct MakeTargetEntry {
    name: String,
    defined_in: Vec<String>,
    visibility: String,
    used_in: Vec<String>,
}

fn parse_curated_targets(root_mk: &str) -> Vec<String> {
    let curated_block = root_mk
        .split("CURATED_TARGETS := \\")
        .nth(1)
        .and_then(|rest| rest.split("\n\nhelp:").next())
        .expect("curated targets block");
    let mut targets = Vec::new();
    for line in curated_block.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let trimmed = trimmed.trim_end_matches('\\').trim();
        if trimmed.is_empty() {
            continue;
        }
        for part in trimmed.split_whitespace() {
            targets.push(part.to_string());
        }
    }
    targets
}

fn parse_make_targets(repo: &Path) -> BTreeMap<String, BTreeSet<String>> {
    let mut out = BTreeMap::<String, BTreeSet<String>>::new();
    let mut files = vec![repo.join("Makefile")];
    for entry in fs::read_dir(repo.join("makefiles")).expect("read makefiles dir") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|v| v.to_str()) == Some("mk") {
            files.push(path);
        }
    }
    files.sort();
    for path in files {
        let rel = path.strip_prefix(repo).expect("relative path");
        let text = fs::read_to_string(&path).expect("read makefile");
        for line in text.lines() {
            let Some((left, right)) = line.split_once(':') else {
                continue;
            };
            if left.is_empty() || left.starts_with('.') || left.contains(' ') {
                continue;
            }
            // Skip variable assignments like "FOO := bar"
            if right.starts_with('=') {
                continue;
            }
            if left
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/'))
            {
                out.entry(left.to_string())
                    .or_default()
                    .insert(rel.display().to_string());
            }
        }
    }
    out
}

fn parse_workflow_make_uses(repo: &Path) -> BTreeMap<String, BTreeSet<String>> {
    let mut out = BTreeMap::<String, BTreeSet<String>>::new();
    let workflows = repo.join(".github/workflows");
    let mut files = Vec::new();
    for entry in fs::read_dir(workflows).expect("read workflows dir") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|v| v.to_str()) == Some("yml") {
            files.push(path);
        }
    }
    files.sort();
    for file in files {
        let rel = file.strip_prefix(repo).expect("relative path");
        let text = fs::read_to_string(&file).expect("read workflow");
        for line in text.lines() {
            let mut rest = line;
            while let Some(idx) = rest.find("make ") {
                let after = &rest[idx + 5..];
                let target: String = after
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/'))
                    .collect();
                if !target.is_empty() {
                    out.entry(target)
                        .or_default()
                        .insert(rel.display().to_string());
                }
                rest = after;
            }
        }
    }
    out
}

#[test]
fn make_target_registry_matches_makefiles_and_curated_surface() {
    let repo = repo_root();
    let registry_path = repo.join("configs/ops/make-target-registry.json");
    let registry: MakeTargetRegistry =
        serde_json::from_str(&fs::read_to_string(registry_path).expect("read registry"))
            .expect("parse registry");
    assert_eq!(registry.schema_version, 1);

    let root_mk = fs::read_to_string(repo.join("makefiles/root.mk")).expect("read root makefile");
    let curated = parse_curated_targets(&root_mk);
    let curated_set: BTreeSet<String> = curated.into_iter().collect();

    let discovered = parse_make_targets(&repo);
    let discovered_set: BTreeSet<String> = discovered.keys().cloned().collect();

    let registry_set: BTreeSet<String> = registry.targets.iter().map(|t| t.name.clone()).collect();
    assert_eq!(
        discovered_set, registry_set,
        "registry target names must exactly match makefile-discovered target names"
    );

    let public_set: BTreeSet<String> = registry
        .targets
        .iter()
        .filter(|entry| entry.visibility == "public")
        .map(|entry| entry.name.clone())
        .collect();
    assert_eq!(
        curated_set, public_set,
        "registry public targets must match CURATED_TARGETS from makefiles/root.mk"
    );

    for entry in &registry.targets {
        let expected_files = discovered
            .get(&entry.name)
            .expect("target discovered from makefiles");
        let actual_files: BTreeSet<String> = entry.defined_in.iter().cloned().collect();
        assert_eq!(
            *expected_files, actual_files,
            "registry defined_in drift for target `{}`",
            entry.name
        );
    }
}

#[test]
fn workflow_make_invocations_are_registered_and_tracked() {
    let repo = repo_root();
    let registry: MakeTargetRegistry = serde_json::from_str(
        &fs::read_to_string(repo.join("configs/ops/make-target-registry.json"))
            .expect("read registry"),
    )
    .expect("parse registry");

    let by_target: BTreeMap<String, &MakeTargetEntry> = registry
        .targets
        .iter()
        .map(|e| (e.name.clone(), e))
        .collect();
    let workflow_uses = parse_workflow_make_uses(&repo);

    for (target, files) in workflow_uses {
        let entry = by_target
            .get(&target)
            .unwrap_or_else(|| panic!("workflow uses unknown make target `{target}`"));
        let tracked: BTreeSet<String> = entry.used_in.iter().cloned().collect();
        for file in files {
            assert!(
                tracked.contains(&file),
                "registry entry `{}` missing workflow usage `{}`",
                target,
                file
            );
        }
    }
}
