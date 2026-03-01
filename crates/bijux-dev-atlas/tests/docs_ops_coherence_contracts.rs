// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use serde_yaml::Value as YamlValue;

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

fn load_json(path: &Path) -> Value {
    serde_json::from_str(&read(path))
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()))
}

fn markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
                    out.push(path);
                }
            }
        }
    }
    out.sort();
    out
}

fn parse_glossary_terms(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with("- `") {
                return None;
            }
            let rest = trimmed.strip_prefix("- `")?;
            let term = rest.split("`:").next()?;
            Some(term.to_string())
        })
        .collect()
}

fn parse_mkdocs_top_level_nav(root: &Path) -> Vec<String> {
    let yaml: YamlValue = serde_yaml::from_str(&read(&root.join("mkdocs.yml")))
        .expect("mkdocs.yml must parse");
    yaml.get("nav")
        .and_then(YamlValue::as_sequence)
        .expect("mkdocs nav")
        .iter()
        .filter_map(|item| item.as_mapping())
        .filter_map(|map| map.keys().next())
        .filter_map(YamlValue::as_str)
        .map(str::to_string)
        .collect()
}

fn parse_nav_contract_list(text: &str, prefix: &str) -> Vec<String> {
    text.lines()
        .find_map(|line| line.trim().strip_prefix(prefix).map(str::to_string))
        .expect("nav contract line")
        .split(',')
        .map(|item| item.trim().trim_matches(|c| c == '`' || c == '.').to_string())
        .collect()
}

fn parse_markdown_table_rows(text: &str) -> Vec<Vec<String>> {
    text.lines()
        .filter(|line| line.trim_start().starts_with('|'))
        .skip(2)
        .map(|line| {
            line.trim()
                .trim_matches('|')
                .split('|')
                .map(|cell| cell.trim().trim_matches('`').to_string())
                .collect::<Vec<_>>()
        })
        .filter(|row| row.len() >= 3)
        .collect()
}

#[test]
fn root_readme_routes_getting_started_to_docs_start_here_only() {
    let root = repo_root();
    let readme = read(&root.join("README.md"));
    assert!(
        readme.contains("docs/start-here.md"),
        "README.md must point to docs/start-here.md as the getting-started entrypoint"
    );
    for forbidden in ["make bootstrap", "make doctor", "make check", "make test"] {
        assert!(
            !readme.contains(forbidden),
            "README.md Quick Start must route to docs/start-here.md instead of embedding `{forbidden}`"
        );
    }
    for forbidden in ["runbook", "rollback-playbook", "incident-playbook"] {
        assert!(
            !readme.to_ascii_lowercase().contains(forbidden),
            "README.md must not embed operational runbook routing: {forbidden}"
        );
    }
}

#[test]
fn operations_docs_reference_existing_canonical_ops_paths() {
    let root = repo_root();
    let mut violations = Vec::new();
    for file in markdown_files(&root.join("docs/operations")) {
        let rel = file.strip_prefix(&root).expect("repo relative").display().to_string();
        let text = read(&file);
        for line in text.lines() {
            if !line.contains("Source-of-truth:") {
                continue;
            }
            for segment in line.split('`') {
                if !(segment.starts_with("ops/") || segment.starts_with("configs/ops/")) {
                    continue;
                }
                if segment.contains('*') || segment.contains("**") {
                    continue;
                }
                let candidate = root.join(segment);
                if !candidate.exists() {
                    violations.push(format!("{rel} references missing path `{segment}`"));
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "operations docs must reference existing canonical ops paths:\n{}",
        violations.join("\n")
    );
}

#[test]
#[ignore = "ops docs mapping pending canonical ops map rewrite"]
fn glossary_covers_ops_public_terms_and_every_term_is_used() {
    let root = repo_root();
    let glossary = read(&root.join("docs/glossary.md"));
    let terms = parse_glossary_terms(&glossary);
    for required in [
        "Release",
        "Dataset",
        "Stack",
        "K8s",
        "Load",
        "E2E",
        "Fixture",
        "Profile",
        "Lane",
    ] {
        assert!(
            terms.contains(required),
            "docs/glossary.md must define `{required}`"
        );
    }

    let mut corpus = String::new();
    for file in markdown_files(&root.join("docs")) {
        let rel = file.strip_prefix(&root).expect("repo relative");
        if rel == Path::new("docs/glossary.md") {
            continue;
        }
        corpus.push_str(&read(&file));
        corpus.push('\n');
    }
    for term in terms {
        assert!(
            corpus.contains(&format!("`{term}`")) || corpus.contains(&term),
            "glossary term must be referenced by at least one other docs page: {term}"
        );
    }
}

#[test]
fn docs_sections_and_ops_pillars_have_one_shared_owner_identity() {
    let root = repo_root();
    let identities = load_json(&root.join("configs/owners/identities.json"))["identities"]
        .as_object()
        .expect("owner identities object")
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();

    let docs_owners_json = load_json(&root.join("docs/owners.json"));
    let docs_owners = docs_owners_json["section_owners"]
        .as_object()
        .expect("docs section owners");
    assert!(
        !docs_owners.is_empty(),
        "docs/owners.json must declare section owners"
    );
    for (section, owner) in docs_owners {
        let owner = owner
            .as_str()
            .unwrap_or_else(|| panic!("docs section owner must be a string: {section}"));
        assert!(
            !owner.is_empty(),
            "docs section owner must be non-empty: {section}"
        );
        assert!(
            identities.contains(owner),
            "docs section owner must exist in configs/owners/identities.json: {section} -> {owner}"
        );
    }

    let pillars_json = load_json(&root.join("ops/inventory/pillars.json"));
    let pillars = pillars_json["pillars"]
        .as_array()
        .expect("ops pillars");
    let mut pillar_ids = BTreeSet::new();
    for pillar in pillars {
        let pillar_id = pillar["id"].as_str().expect("pillar id");
        let owner = pillar["owner"].as_str().expect("pillar owner");
        assert!(pillar_ids.insert(pillar_id.to_string()), "duplicate pillar id: {pillar_id}");
        assert!(
            identities.contains(owner),
            "ops pillar owner must exist in configs/owners/identities.json: {pillar_id} -> {owner}"
        );
    }

    let ops_owners_json = load_json(&root.join("ops/inventory/owners.json"));
    let ops_owners = ops_owners_json["areas"]
        .as_object()
        .expect("ops areas");
    for (area, owner) in ops_owners {
        let owner = owner
            .as_str()
            .unwrap_or_else(|| panic!("ops owner must be a string: {area}"));
        assert!(
            !owner.is_empty(),
            "ops owner must be non-empty: {area}"
        );
        assert!(
            identities.contains(owner),
            "ops owner must exist in configs/owners/identities.json: {area} -> {owner}"
        );
    }
}

#[test]
fn codeowners_matches_shared_docs_and_ops_owner_handles() {
    let root = repo_root();
    let identities_json = load_json(&root.join("configs/owners/identities.json"));
    let identities = identities_json["identities"]
        .as_object()
        .expect("owner identities");
    let codeowners = read(&root.join(".github/CODEOWNERS"));

    let required_patterns = [
        ("/docs/", "docs-governance"),
        ("/ops/stack/", "bijux-atlas-operations"),
        ("/ops/k8s/", "bijux-atlas-operations"),
        ("/ops/e2e/", "bijux-atlas-operations"),
        ("/ops/observe/", "bijux-atlas-observability"),
        ("/ops/load/", "bijux-atlas-performance"),
        ("/ops/run/", "bijux-atlas-operations"),
    ];
    for (pattern, owner_key) in required_patterns {
        let handles = identities[owner_key]["github_handles"]
            .as_array()
            .expect("github_handles");
        let expected = handles
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            codeowners.contains(&format!("{pattern} {expected}")),
            "CODEOWNERS must map `{pattern}` to the shared owner handles for `{owner_key}`"
        );
    }
}

#[test]
fn mkdocs_nav_matches_docs_nav_contract() {
    let root = repo_root();
    let nav = parse_mkdocs_top_level_nav(&root);
    let nav_contract = read(&root.join("docs/_internal/nav/index.md"));
    let declared_names = parse_nav_contract_list(
        &nav_contract,
        "- Top-level navigation labels are fixed: ",
    );
    let declared_order = parse_nav_contract_list(
        &nav_contract,
        "- Top-level navigation order is fixed: ",
    )
    .into_iter()
    .flat_map(|item| item.split(" -> ").map(str::to_string).collect::<Vec<_>>())
    .collect::<Vec<_>>();

    assert_eq!(
        nav, declared_names,
        "mkdocs top-level nav labels must match docs/_internal/nav/index.md"
    );
    assert_eq!(
        nav, declared_order,
        "mkdocs top-level nav order must match docs/_internal/nav/index.md"
    );
}

#[test]
fn ops_map_covers_every_pillar_with_one_docs_entrypoint() {
    let root = repo_root();
    let ops_map = read(&root.join("docs/operations/ops-map.md"));
    let rows = parse_markdown_table_rows(&ops_map);
    let mapped = rows
        .iter()
        .map(|row| row[0].clone())
        .collect::<BTreeSet<_>>();
    let pillars_json = load_json(&root.join("ops/inventory/pillars.json"));
    let pillars = pillars_json["pillars"]
        .as_array()
        .expect("pillars");
    let expected = pillars
        .iter()
        .filter_map(|row| row["id"].as_str())
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        mapped, expected,
        "docs/operations/ops-map.md must map every ops pillar exactly once"
    );
    for row in rows {
        let surface = &row[1];
        let docs_entry = &row[2];
        assert!(root.join("ops").join(surface.trim_start_matches("ops/")).exists() || root.join(surface).exists(),
            "ops map surface path must exist: {surface}");
        assert!(root.join("docs/operations").join(docs_entry).exists(),
            "ops map docs entry must exist: docs/operations/{docs_entry}");
    }
}

#[test]
fn runbooks_reference_known_alert_and_slo_catalog_entries() {
    let root = repo_root();
    let alert_catalog = load_json(&root.join("ops/observe/alert-catalog.json"));
    let alerts = alert_catalog["alerts"]
        .as_array()
        .expect("alerts")
        .iter()
        .filter_map(|row| row["id"].as_str())
        .collect::<BTreeSet<_>>();
    let slo_catalog = load_json(&root.join("ops/observe/slo-definitions.json"));
    let slos = slo_catalog["slos"]
        .as_array()
        .expect("slos")
        .iter()
        .filter_map(|row| row["id"].as_str())
        .collect::<BTreeSet<_>>();

    let alert_re = regex::Regex::new(r"`(BijuxAtlas[A-Za-z0-9]+)`").expect("alert regex");
    let slo_re = regex::Regex::new(r"`(api\.[a-z0-9_]+)`").expect("slo regex");
    let mut violations = Vec::new();

    for path in markdown_files(&root.join("docs/operations/runbooks")) {
        let rel = path.strip_prefix(&root).expect("repo relative").display().to_string();
        if rel.ends_with("INDEX.md") {
            continue;
        }
        let text = read(&path);
        for caps in alert_re.captures_iter(&text) {
            let alert = caps.get(1).expect("alert id").as_str();
            if !alerts.contains(alert) {
                violations.push(format!("{rel} references unknown alert `{alert}`"));
            }
        }
        for caps in slo_re.captures_iter(&text) {
            let slo = caps.get(1).expect("slo id").as_str();
            if !slos.contains(slo) {
                violations.push(format!("{rel} references unknown slo `{slo}`"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "operations runbooks must reference alerts and SLOs from canonical catalogs:\n{}",
        violations.join("\n")
    );
}

#[test]
fn operations_docs_bind_kind_fixture_and_promotion_pages_to_canonical_ops_paths() {
    let root = repo_root();
    let cases = [
        (
            "docs/operations/kind-profile.md",
            [
                "ops/stack/profiles.json",
                "ops/stack/kind/cluster-dev.yaml",
                "ops/stack/kind/cluster-perf.yaml",
                "ops/stack/kind/cluster-small.yaml",
            ]
            .as_slice(),
        ),
        (
            "docs/operations/fixture-dataset-ingest.md",
            [
                "ops/datasets/manifest.json",
                "ops/datasets/generated/fixture-inventory.json",
                "ops/datasets/fixture-policy.json",
            ]
            .as_slice(),
        ),
        (
            "docs/operations/promotion-record.md",
            [
                "ops/datasets/promotion-rules.json",
                "ops/datasets/rollback-policy.json",
                "ops/_generated.example/fixture-drift-report.json",
            ]
            .as_slice(),
        ),
    ];
    for (page, required) in cases {
        let text = read(&root.join(page));
        for rel in required {
            assert!(text.contains(rel), "{page} must reference `{rel}`");
            assert!(root.join(rel).exists(), "{page} reference must exist: {rel}");
        }
    }
}

#[test]
fn docs_must_not_invent_ops_steps_outside_control_plane_and_manifest_surfaces() {
    let root = repo_root();
    let surfaces = load_json(&root.join("ops/inventory/surfaces.json"));
    let commands = surfaces["bijux-dev-atlas_commands"]
        .as_array()
        .expect("ops commands")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    let allowed_make = surfaces["entrypoints"]
        .as_array()
        .expect("entrypoints")
        .iter()
        .filter_map(Value::as_str)
        .map(|name| format!("make {name}"))
        .collect::<BTreeSet<_>>();
    let mut violations = Vec::new();
    for path in markdown_files(&root.join("docs/operations")) {
        let rel = path.strip_prefix(&root).expect("repo relative").display().to_string();
        let text = read(&path);
        for line in text.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with("bijux dev atlas ops ") && !trimmed.starts_with("make ops-") {
                continue;
            }
            let exact = trimmed.trim_matches('`');
            let known_cli = commands.iter().any(|cmd| exact.starts_with(cmd));
            let known_make = allowed_make.iter().any(|cmd| exact.starts_with(cmd));
            if !known_cli && !known_make {
                violations.push(format!("{rel} invents non-canonical ops command `{exact}`"));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "operations docs must use canonical control-plane commands only:\n{}",
        violations.join("\n")
    );
}

#[test]
#[ignore = "ops docs mapping pending canonical ops map rewrite"]
fn ops_help_output_lists_pillars_and_doc_entrypoints() {
    let root = repo_root();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["ops", "--help"])
        .output()
        .expect("ops --help");
    assert!(output.status.success(), "ops --help must succeed");
    let help = String::from_utf8(output.stdout).expect("utf8");
    for required in [
        "Ops Pillars And Docs Entrypoints:",
        "inventory -> docs/operations/reference/ops-surface.md",
        "k8s -> docs/operations/k8s/index.md",
        "datasets -> docs/operations/datasets.md",
        "report -> docs/operations/unified-report.md",
    ] {
        assert!(help.contains(required), "ops --help missing `{required}`");
    }
}

#[test]
fn ops_contract_ids_stay_explicit_and_non_range_based() {
    let root = repo_root();
    let snapshot = load_json(&root.join("ops/_generated.example/contracts-registry-snapshot.json"));
    let ids = snapshot["contracts"]
        .as_array()
        .expect("contracts")
        .iter()
        .filter_map(|row| row["id"].as_str());
    let range_re = regex::Regex::new(r"OPS-[A-Z0-9-]+-\d{3}-\d{3}$").expect("range regex");
    for id in ids {
        assert!(
            !range_re.is_match(id),
            "ops contract ids must stay explicit and non-range-based: {id}"
        );
    }
}
