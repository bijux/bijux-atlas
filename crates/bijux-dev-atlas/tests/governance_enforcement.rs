// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use regex::Regex;
use serde::Deserialize;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[derive(Debug, Deserialize)]
struct RegistryFile {
    checks: Vec<RegistryCheck>,
    tags: Option<RegistryTags>,
}

#[derive(Debug, Deserialize)]
struct RegistryCheck {
    id: String,
    domain: String,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RegistryTags {
    vocabulary: Vec<String>,
}

#[test]
fn registry_checks_have_stable_ids_tags_and_inventory_mapping() {
    let root = workspace_root();
    let registry_path = root.join("ops/inventory/registry.toml");
    let text = fs::read_to_string(&registry_path).expect("registry");
    let registry: RegistryFile = toml::from_str(&text).expect("valid registry");

    let allowed_tags = registry
        .tags
        .as_ref()
        .map(|tags| tags.vocabulary.iter().cloned().collect::<BTreeSet<_>>())
        .unwrap_or_default();
    let id_pattern = Regex::new(r"^checks_[a-z0-9_]+$").expect("id regex");
    let mut ids = BTreeSet::new();
    for check in registry.checks {
        assert!(
            !check.id.trim().is_empty(),
            "registry check id must be non-empty"
        );
        assert!(
            id_pattern.is_match(&check.id),
            "registry check id must match `checks_[a-z0-9_]+`: {}",
            check.id
        );
        assert!(
            ids.insert(check.id.clone()),
            "registry check id must be unique: {}",
            check.id
        );
        assert!(
            check.tags.iter().any(|tag| !tag.trim().is_empty()),
            "registry check `{}` must define at least one tag",
            check.id
        );
        for tag in &check.tags {
            assert!(
                allowed_tags.is_empty() || allowed_tags.contains(tag),
                "registry check `{}` uses tag `{}` not present in tags vocabulary",
                check.id,
                tag
            );
        }
        assert!(
            matches!(
                check.domain.as_str(),
                "root"
                    | "workflows"
                    | "configs"
                    | "docker"
                    | "crates"
                    | "ops"
                    | "repo"
                    | "docs"
                    | "make"
            ),
            "registry check `{}` has invalid domain mapping `{}`",
            check.id,
            check.domain
        );
    }
}

#[test]
fn generated_report_examples_reference_existing_report_schemas() {
    let root = workspace_root();
    let required_pairs = [
        (
            "ops/_generated.example/report.unified.example.json",
            "ops/schema/report/unified.schema.json",
        ),
        (
            "ops/_generated.example/stack-health-report.example.json",
            "ops/schema/report/stack-health-report.schema.json",
        ),
        (
            "ops/_generated.example/evidence-gap-report.json",
            "ops/schema/report/evidence-gap-report.schema.json",
        ),
    ];
    for (report_rel, schema_rel) in required_pairs {
        let report_path = root.join(report_rel);
        let schema_path = root.join(schema_rel);
        assert!(
            report_path.exists(),
            "missing generated example {}",
            report_path.display()
        );
        assert!(
            schema_path.exists(),
            "missing report schema {}",
            schema_path.display()
        );
    }
}
