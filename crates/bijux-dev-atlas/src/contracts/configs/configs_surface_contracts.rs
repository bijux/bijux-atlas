fn test_configs_037_no_latest_refs(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => return fail("CONFIGS-037", "configs.supplychain.no_latest", REGISTRY_PATH, err),
    };
    let mut violations = Vec::new();
    for file in config_files_without_exclusions(&index) {
        let path = ctx.repo_root.join(&file);
        let Ok(contents) = std::fs::read_to_string(&path) else {
            continue;
        };
        if contents.contains(":latest") {
            violations.push(violation(
                "CONFIGS-037",
                "configs.supplychain.no_latest",
                &file,
                "forbidden mutable `:latest` reference found in config surface",
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

pub fn contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
    Ok(vec![
        contract(
            "CONFIGS-001",
            "configs root keeps only declared root files",
            "configs.root.only_root_docs",
            "configs root file surface matches registry",
            test_configs_001_root_surface,
        ),
        contract(
            "CONFIGS-002",
            "configs files are documented by the registry",
            "configs.registry.no_undocumented_files",
            "registry covers every config file",
            test_configs_002_no_undocumented_files,
        ),
        contract(
            "CONFIGS-003",
            "configs path depth stays within budget",
            "configs.layout.depth_budget",
            "configs depth budget stays within registry limits",
            test_configs_003_depth_budget,
        ),
        contract(
            "CONFIGS-004",
            "configs internal surfaces stay explicitly internal",
            "configs.naming.internal_surface",
            "internal configs are not exposed as public",
            test_configs_004_internal_naming,
        ),
        contract(
            "CONFIGS-005",
            "configs groups declare owners",
            "configs.registry.owner_complete",
            "each configs group has an owner",
            test_configs_005_owner_complete,
        ),
        contract(
            "CONFIGS-006",
            "configs groups declare schema coverage",
            "configs.schema.coverage",
            "json-bearing groups declare real schema files",
            test_configs_006_schema_coverage,
        ),
        contract(
            "CONFIGS-007",
            "configs lockfile pairs stay complete",
            "configs.lockfiles.required_pairs",
            "tool dependency configs keep lockfiles",
            test_configs_007_lockfiles,
        ),
        contract(
            "CONFIGS-008",
            "configs registry avoids duplicate group ownership",
            "configs.registry.no_overlap",
            "no config file is claimed by multiple groups",
            test_configs_008_no_overlap,
        ),
        contract(
            "CONFIGS-009",
            "generated config surfaces stay separate from authored files",
            "configs.generated.authored_boundary",
            "generated config patterns stay under _generated surfaces",
            test_configs_009_generated_boundary,
        ),
        contract(
            "CONFIGS-010",
            "configs contracts doc mirrors executable checks",
            "configs.contracts.no_policy_theater",
            "contract docs match enforced config checks",
            test_configs_010_no_policy_theater,
        ),
        contract(
            "CONFIGS-011",
            "configs registry keeps a complete root surface",
            "configs.registry.complete_surface",
            "registry keeps the root docs and manifest visible",
            test_configs_011_registry_complete,
        ),
        contract(
            "CONFIGS-012",
            "configs registry leaves no orphan files",
            "configs.registry.no_orphans",
            "all config files belong to the registry",
            test_configs_012_no_orphans,
        ),
        contract(
            "CONFIGS-013",
            "configs registry leaves no dead entries",
            "configs.registry.no_dead_entries",
            "all registry patterns and exclusions match real files",
            test_configs_013_no_dead_entries,
        ),
        contract(
            "CONFIGS-014",
            "configs group count stays within budget",
            "configs.registry.group_budget",
            "configs group count stays under max_groups",
            test_configs_014_group_budget,
        ),
        contract(
            "CONFIGS-015",
            "configs group paths stay within group depth budget",
            "configs.registry.group_depth_budget",
            "config files do not exceed per-group depth budget",
            test_configs_015_group_depth_budget,
        ),
        contract(
            "CONFIGS-016",
            "configs files declare exactly one visibility class",
            "configs.registry.visibility_classification",
            "each config file maps to public, internal, or generated",
            test_configs_016_visibility_classification,
        ),
        contract(
            "CONFIGS-017",
            "configs groups declare tool entrypoints",
            "configs.registry.tool_entrypoints",
            "each configs group declares consuming command entrypoints",
            test_configs_017_tool_entrypoints,
        ),
        contract(
            "CONFIGS-018",
            "configs groups declare schema ownership",
            "configs.registry.schema_owner",
            "schema files map to an explicit schema owner",
            test_configs_018_schema_owner,
        ),
        contract(
            "CONFIGS-019",
            "configs groups declare lifecycle metadata",
            "configs.registry.lifecycle",
            "each configs group declares stability-tier lifecycle metadata",
            test_configs_019_lifecycle,
        ),
        contract(
            "CONFIGS-020",
            "configs generated index stays deterministic",
            "configs.generated_index.deterministic",
            "generated configs index renders deterministically",
            test_configs_020_generated_index_deterministic,
        ),
        contract(
            "CONFIGS-021",
            "configs generated index matches committed output",
            "configs.generated_index.committed_match",
            "committed generated configs index matches the registry render",
            test_configs_021_generated_index_matches_committed,
        ),
        contract(
            "CONFIGS-022",
            "configs json surfaces parse cleanly",
            "configs.parse.json",
            "json and jsonc config files parse successfully",
            test_configs_022_json_configs_parse,
        ),
        contract(
            "CONFIGS-023",
            "configs yaml surfaces parse cleanly",
            "configs.parse.yaml",
            "yaml config files parse successfully",
            test_configs_023_yaml_configs_parse,
        ),
        contract(
            "CONFIGS-024",
            "configs toml surfaces parse cleanly",
            "configs.parse.toml",
            "toml config files parse successfully",
            test_configs_024_toml_configs_parse,
        ),
        contract(
            "CONFIGS-025",
            "configs text surfaces avoid whitespace drift",
            "configs.text.hygiene",
            "text config files avoid trailing whitespace drift",
            test_configs_025_text_hygiene,
        ),
        contract(
            "CONFIGS-026",
            "configs docs directory forbids nested markdown",
            "configs.docs.no_nested_markdown",
            "configs docs keeps tooling inputs only",
            test_configs_026_docs_markdown_removed,
        ),
        contract(
            "CONFIGS-027",
            "configs docs directory stays tooling only",
            "configs.docs.tooling_surface",
            "configs docs files stay within the declared tooling surface",
            test_configs_027_docs_tooling_surface,
        ),
        contract(
            "CONFIGS-028",
            "configs owner map stays aligned with the registry",
            "configs.owners.group_alignment",
            "configs owner map matches the declared group owners",
            test_configs_028_owner_map_alignment,
        ),
        contract(
            "CONFIGS-029",
            "configs consumer map stays aligned with the registry",
            "configs.consumers.group_alignment",
            "configs consumer map matches the declared groups",
            test_configs_029_consumer_map_alignment,
        ),
        contract(
            "CONFIGS-030",
            "configs public files declare file-level consumers",
            "configs.consumers.file_coverage",
            "public and generated config files have per-file consumer coverage",
            test_configs_030_file_consumer_coverage,
        ),
        contract(
            "CONFIGS-031",
            "configs json files declare file-level schema coverage",
            "configs.schemas.file_coverage",
            "root, public, and generated json configs map to declared schemas",
            test_configs_031_schema_map_coverage,
        ),
        contract(
            "CONFIGS-032",
            "configs root json surfaces stay canonical",
            "configs.json.canonical_root_surface",
            "root authority json files use canonical stable formatting",
            test_configs_032_root_json_canonical,
        ),
        contract(
            "CONFIGS-033",
            "configs schema index matches committed output",
            "configs.schema.index_committed_match",
            "committed schema index matches the canonical schema map render",
            test_configs_033_schema_index_matches_committed,
        ),
        contract(
            "CONFIGS-034",
            "configs input schemas stay referenced",
            "configs.schema.no_orphan_inputs",
            "every input schema is referenced by a governed config mapping",
            test_configs_034_no_orphan_input_schemas,
        ),
        contract(
            "CONFIGS-035",
            "configs schema versioning policy stays complete",
            "configs.schema.versioning_policy",
            "governed public schemas stay covered by the schema versioning policy",
            test_configs_035_schema_versioning_policy,
        ),
        contract(
            "CONFIGS-036",
            "configs exclusions carry approval and expiry metadata",
            "configs.exclusions.governed_metadata",
            "registry exclusions declare approver and expiry metadata",
            test_configs_036_exclusion_governance,
        ),
        contract(
            "CONFIGS-037",
            "configs surfaces forbid mutable latest-tag references",
            "configs.supplychain.no_latest",
            "config files may not embed mutable :latest references",
            test_configs_037_no_latest_refs,
        ),
    ])
}

fn contract(
    id: &'static str,
    title: &'static str,
    test_id: &'static str,
    test_title: &'static str,
    run: fn(&RunContext) -> TestResult,
) -> Contract {
    Contract {
        id: ContractId(id.to_string()),
        title,
        tests: vec![TestCase {
            id: TestId(test_id.to_string()),
            title: test_title,
            kind: TestKind::Pure,
            run,
        }],
    }
}

pub fn contract_explain(contract_id: &str) -> String {
    match contract_id {
        "CONFIGS-001" => "The configs root is a tiny authority surface: README.md, CONTRACT.md, and the legacy inventory pointer only.".to_string(),
        "CONFIGS-002" => "Every config file must be covered by the canonical configs registry so filesystem drift is visible.".to_string(),
        "CONFIGS-003" => "Configs path depth stays within an explicit budget to avoid unreviewable nesting.".to_string(),
        "CONFIGS-004" => "Internal config surfaces must stay internal and cannot leak into public classifications.".to_string(),
        "CONFIGS-005" => "Every configs group needs an explicit owner in the registry.".to_string(),
        "CONFIGS-006" => "JSON-bearing config groups must declare real schema files so validation has a source of truth.".to_string(),
        "CONFIGS-007" => "Pinned tool dependency manifests require committed lockfiles.".to_string(),
        "CONFIGS-008" => "A config file can only have one group owner in the registry.".to_string(),
        "CONFIGS-009" => "Generated configs stay under explicit _generated surfaces instead of mixing with authored files.".to_string(),
        "CONFIGS-010" => "Configs contracts docs must match the executable checks; documentation alone is not evidence.".to_string(),
        "CONFIGS-011" => "The configs registry must describe the root surface completely and deterministically.".to_string(),
        "CONFIGS-012" => "No config file may exist outside the registry.".to_string(),
        "CONFIGS-013" => "Registry patterns and exclusions must resolve to real files, not stale entries.".to_string(),
        "CONFIGS-014" => "Configs groups stay within an explicit group-count budget.".to_string(),
        "CONFIGS-015" => "Each configs group stays within a bounded path depth budget.".to_string(),
        "CONFIGS-016" => "Each config file must map to exactly one visibility class and each group declares its stability.".to_string(),
        "CONFIGS-017" => "Every configs group must declare the commands that consume that configuration surface.".to_string(),
        "CONFIGS-018" => "Schema-bearing groups must declare an explicit schema owner and real schema files.".to_string(),
        "CONFIGS-019" => "Each configs group declares stable lifecycle metadata through owner, schema owner, and stability.".to_string(),
        "CONFIGS-020" => "The generated configs index must be deterministic from the registry.".to_string(),
        "CONFIGS-021" => "The committed generated configs index must match the canonical registry render.".to_string(),
        "CONFIGS-022" => "JSON and JSONC config files must parse successfully.".to_string(),
        "CONFIGS-023" => "YAML config files must parse successfully.".to_string(),
        "CONFIGS-024" => "TOML config files must parse successfully.".to_string(),
        "CONFIGS-025" => "Config text files must not accumulate trailing whitespace drift.".to_string(),
        "CONFIGS-026" => "The configs/docs directory must not contain narrative markdown.".to_string(),
        "CONFIGS-027" => "The configs/docs directory must stay within its declared tooling file surface.".to_string(),
        "CONFIGS-028" => "The canonical configs owner map must match the registry group owners.".to_string(),
        "CONFIGS-029" => "The canonical configs consumer map must cover the registry groups.".to_string(),
        "CONFIGS-030" => "Every public or generated config file must have explicit file-level consumer coverage in configs/CONSUMERS.json.".to_string(),
        "CONFIGS-031" => "Root, public, and generated JSON or JSONC configs must map to explicit schema coverage in configs/SCHEMAS.json.".to_string(),
        "CONFIGS-032" => "The root configs authority JSON files and generated configs index must stay in canonical sorted pretty JSON form.".to_string(),
        "CONFIGS-033" => "The committed configs schema index must match the canonical render from configs/SCHEMAS.json and the schema directories.".to_string(),
        "CONFIGS-034" => "Every input schema under configs/schema must be referenced by at least one governed config mapping in configs/SCHEMAS.json.".to_string(),
        "CONFIGS-035" => "Every governed public schema file must be listed in configs/schema/versioning-policy.json with the supported compatibility and versioning rules.".to_string(),
        "CONFIGS-036" => "Every registry exclusion must carry explicit approver and expiry metadata so allowlists stay reviewable.".to_string(),
        "CONFIGS-037" => "Config surfaces may not embed mutable latest-tag references.".to_string(),
        _ => "Fix the listed violations and rerun `bijux dev atlas contracts configs`.".to_string(),
    }
}

pub fn explain_payload(repo_root: &Path, file: &str) -> Result<serde_json::Value, String> {
    let index = registry_index(repo_root)?;
    let owners = read_owners(repo_root)?;
    let consumers = read_consumers(repo_root)?;
    let schemas = read_schemas(repo_root)?;
    let normalized = file.replace('\\', "/");
    if index.root_files.contains(&normalized) {
        return Ok(serde_json::json!({
            "schema_version": 1,
            "kind": "configs_explain",
            "path": normalized,
            "group": serde_json::Value::Null,
            "visibility": "root",
            "owner": serde_json::Value::Null,
            "consumers": matching_file_consumers(&consumers, &normalized),
            "schema": matched_schema_path(&schemas, &normalized),
            "schema_owner": serde_json::Value::Null,
            "stability": "stable",
            "tool_entrypoints": [],
            "summary": "root configs authority file"
        }));
    }
    for exclusion in &index.registry.exclusions {
        if wildcard_match(&exclusion.pattern, &normalized) {
            return Ok(serde_json::json!({
                "schema_version": 1,
                "kind": "configs_explain",
                "path": normalized,
                "group": serde_json::Value::Null,
                "visibility": "excluded",
                "owner": serde_json::Value::Null,
                "consumers": [],
                "schema": serde_json::Value::Null,
                "schema_owner": serde_json::Value::Null,
                "stability": serde_json::Value::Null,
                "tool_entrypoints": [],
                "summary": exclusion.reason
            }));
        }
    }
    for group in &index.registry.groups {
        let visibility = if matches_any(group.public_files.iter(), &normalized) {
            Some("public")
        } else if matches_any(group.internal_files.iter(), &normalized) {
            Some("internal")
        } else if matches_any(group.generated_files.iter(), &normalized) {
            Some("generated")
        } else {
            None
        };
        if let Some(visibility) = visibility {
            let file_consumers = matching_file_consumers(&consumers, &normalized);
            let effective_consumers = if file_consumers.is_empty() {
                consumers
                    .groups
                    .get(&group.name)
                    .cloned()
                    .unwrap_or_default()
            } else {
                file_consumers
            };
            return Ok(serde_json::json!({
                "schema_version": 1,
                "kind": "configs_explain",
                "path": normalized,
                "group": group.name,
                "visibility": visibility,
                "owner": owners.groups.get(&group.name).cloned().unwrap_or_else(|| group.owner.clone()),
                "consumers": effective_consumers,
                "schema": matched_schema_path(&schemas, &normalized),
                "schema_owner": group.schema_owner,
                "stability": group.stability,
                "tool_entrypoints": group.tool_entrypoints,
                "summary": format!("configs group `{}` {} file", group.name, visibility)
            }));
        }
    }
    Err(format!(
        "config path `{normalized}` is not covered by configs/inventory/configs.json"
    ))
}

pub fn contract_gate_command(_contract_id: &str) -> &'static str {
    "bijux dev atlas contracts configs --mode static"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace")
            .parent()
            .expect("repo")
            .to_path_buf()
    }

    #[test]
    fn registry_index_parses_and_exposes_groups() {
        let index = registry_index(&repo_root()).expect("registry");
        assert_eq!(index.registry.schema_version, 1);
        assert!(index
            .registry
            .groups
            .iter()
            .any(|group| group.name == "inventory"));
        assert!(index
            .registry
            .groups
            .iter()
            .any(|group| group.name == "inventory"
                && group
                    .tool_entrypoints
                    .iter()
                    .any(|entry| entry == "bijux dev atlas configs list")));
    }

    #[test]
    fn generated_index_render_is_stable() {
        let root = repo_root();
        let first = generated_index_json(&root).expect("first");
        let second = generated_index_json(&root).expect("second");
        assert_eq!(first, second);
        let groups = first["groups"].as_array().expect("groups");
        assert!(!groups.is_empty());
    }

    #[test]
    fn wildcard_match_supports_double_star_segments() {
        assert!(wildcard_match(
            "configs/openapi/**/*.json",
            "configs/openapi/v1/openapi.generated.json"
        ));
        assert!(wildcard_match(
            "configs/docs/.vale/styles/**",
            "configs/docs/.vale/styles/Bijux/terminology.yml"
        ));
        assert!(!wildcard_match(
            "configs/docs/*.json",
            "configs/docs/schema-validation.md"
        ));
    }

    #[test]
    fn explain_payload_returns_group_metadata() {
        let payload =
            explain_payload(&repo_root(), "configs/rust/rustfmt.toml").expect("explain payload");
        assert_eq!(payload["group"].as_str(), Some("rust"));
        assert_eq!(payload["visibility"].as_str(), Some("public"));
        assert_eq!(payload["owner"].as_str(), Some("rust-foundation"));
        assert!(payload["consumers"]
            .as_array()
            .is_some_and(|rows| !rows.is_empty()));
        assert!(payload["schema"].is_null());
    }

    #[test]
    fn explain_payload_returns_schema_for_json_file() {
        let payload = explain_payload(&repo_root(), "configs/inventory/configs.json")
            .expect("explain payload");
        assert_eq!(
            payload["schema"].as_str(),
            Some("configs/contracts/inventory-configs.schema.json")
        );
    }

    #[test]
    fn contract_surface_registry_parses_and_covers_cfg_ids() {
        let surface = read_contract_surface(&repo_root()).expect("contract surface");
        assert_eq!(surface.schema_version, 1);
        assert_eq!(surface.domain, "configs");
        assert_eq!(surface.contracts.len(), 39);
        assert!(surface.contracts.iter().any(|row| row.id == "CFG-001"));
        assert!(surface
            .contracts
            .iter()
            .any(|row| row.enforced_by.test_id == "configs.schemas.file_coverage"));
    }

    #[test]
    fn cfg_contract_coverage_payload_is_stable() {
        let payload = cfg_contract_coverage_payload(&repo_root()).expect("coverage payload");
        assert_eq!(payload["contract_count"].as_u64(), Some(39));
        assert!(payload["mapped_checks"].as_u64().is_some());
        assert!(payload["total_checks"].as_u64().is_some());
        assert!(payload["coverage_pct"].as_u64().is_some());
        assert!(payload["registry_sha256"].as_str().is_some());
    }
}
