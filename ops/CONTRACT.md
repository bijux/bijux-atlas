# Ops Contract

- Owner: `bijux-atlas-operations`
- Enforced by: `bijux dev atlas contracts ops`

## Naming

- Contract ID: `OPS-<PILLAR>-NNN`
- Test ID: `ops.<pillar>.<topic>[.<name>]`

## Contract Registry

### Pillar: datasets

#### OPS-DATASETS-001 datasets manifest lock contract

Tests:
- `ops.datasets.manifest_and_lock_consistent` (static, Pure): dataset manifest and lock ids are consistent

#### OPS-DATASETS-002 datasets fixture inventory contract

Tests:
- `ops.datasets.fixture_inventory_matches_disk` (static, Pure): fixture inventory matches fixture directories and references

#### OPS-DATASETS-003 datasets fixture drift promotion contract

Tests:
- `ops.datasets.no_fixture_drift_without_promotion_record` (static, Pure): fixture drift requires explicit promotion rule coverage

#### OPS-DATASETS-004 datasets release diff determinism contract

Tests:
- `ops.datasets.release_diff_fixtures_deterministic` (static, Pure): release-diff fixture lock and golden payloads are deterministic

#### OPS-DATASETS-005 datasets qc metadata summary contract

Tests:
- `ops.datasets.qc_metadata_and_golden_valid` (static, Pure): qc metadata and summary golden are parseable and linked

#### OPS-DATASETS-006 datasets rollback policy contract

Tests:
- `ops.datasets.rollback_policy_exists_valid` (static, Pure): rollback policy exists and includes required rollback structure

#### OPS-DATASETS-007 datasets promotion rules contract

Tests:
- `ops.datasets.promotion_rules_exists_valid` (static, Pure): promotion rules exist with canonical environment coverage and pin references

#### OPS-DATASETS-008 datasets consumer interface contract

Tests:
- `ops.datasets.consumer_list_consistent_with_runtime_queries` (static, Pure): consumer list interfaces resolve to runnable repository paths

#### OPS-DATASETS-009 datasets freeze policy contract

Tests:
- `ops.datasets.freeze_policy_exists_enforced` (static, Pure): freeze policy enforces append-only fixture immutability

#### OPS-DATASETS-010 datasets store layout contract

Tests:
- `ops.datasets.dataset_store_layout_contract_enforced` (static, Pure): dataset ids and fixture paths follow canonical store layout

### Pillar: e2e

#### OPS-E2E-001 e2e suites reference contract

Tests:
- `ops.e2e.suites_reference_real_scenarios` (static, Pure): e2e suites reference concrete scenario ids and runnable entrypoints

#### OPS-E2E-002 e2e smoke manifest contract

Tests:
- `ops.e2e.smoke_manifest_valid` (static, Pure): smoke manifest is structured and points to existing lock

#### OPS-E2E-003 e2e fixtures lock contract

Tests:
- `ops.e2e.fixtures_lock_matches_allowlist` (static, Pure): fixtures lock digest and fixture files match allowlist policy

#### OPS-E2E-004 e2e realdata snapshot contract

Tests:
- `ops.e2e.realdata_snapshots_schema_valid_and_pinned` (static, Pure): realdata snapshots are parseable and pinned to canonical queries

#### OPS-E2E-005 e2e taxonomy coverage contract

Tests:
- `ops.e2e.taxonomy_covers_scenarios` (static, Pure): taxonomy categories cover canonical scenario classification

#### OPS-E2E-006 e2e reproducibility enforcement contract

Tests:
- `ops.e2e.reproducibility_policy_enforced` (static, Pure): reproducibility policy checks and deterministic summary ordering are enforced

#### OPS-E2E-007 e2e coverage matrix determinism contract

Tests:
- `ops.e2e.coverage_matrix_deterministic` (static, Pure): coverage matrix rows and coverage sets are complete and deterministic

#### OPS-E2E-008 e2e realdata scenario registry contract

Tests:
- `ops.e2e.realdata_registry_and_snapshots_valid` (static, Pure): realdata scenarios and snapshots are structurally valid and runnable

#### OPS-E2E-009 e2e surface artifact boundary contract

Tests:
- `ops.e2e.no_stray_e2e_artifacts` (static, Pure): e2e root contains only declared artifact directories and files

#### OPS-E2E-010 e2e summary schema contract

Tests:
- `ops.e2e.summary_schema_valid` (static, Pure): e2e summary is schema-valid and aligned with suite/scenario registries

#### OPS-E2E-E-001 e2e effect smoke suite contract

Tests:
- `ops.e2e.effect.smoke_suite_passes_contract` (effect, Subprocess): effect lane requires smoke suite declaration in e2e suite registry

#### OPS-E2E-E-002 e2e effect realdata suite contract

Tests:
- `ops.e2e.effect.realdata_scenario_passes_contract` (effect, Subprocess): effect lane requires non-empty realdata scenario contract set

### Pillar: env

#### OPS-ENV-001 environment overlay schema contract

Tests:
- `ops.env.overlays_schema_valid` (static, Pure): all required environment overlays are structurally valid

#### OPS-ENV-002 environment profile completeness contract

Tests:
- `ops.env.profiles_complete` (static, Pure): base/ci/dev/prod overlays exist and match profile identity

#### OPS-ENV-003 environment key strictness contract

Tests:
- `ops.env.no_unknown_keys` (static, Pure): environment overlays reject unknown keys

#### OPS-ENV-004 environment overlay merge determinism contract

Tests:
- `ops.env.overlay_merge_deterministic` (static, Pure): overlay merge with identical inputs is deterministic across profiles

#### OPS-ENV-005 environment prod safety toggles contract

Tests:
- `ops.env.prod_forbids_dev_toggles` (static, Pure): prod overlay forbids dev-only effect toggles and unrestricted network

#### OPS-ENV-006 environment ci effect restriction contract

Tests:
- `ops.env.ci_restricts_effects` (static, Pure): ci overlay disables subprocess effects and keeps restricted network mode

#### OPS-ENV-007 environment base defaults contract

Tests:
- `ops.env.base_overlay_required_defaults` (static, Pure): base overlay defines required default keys for all profiles

#### OPS-ENV-008 environment overlay key stability contract

Tests:
- `ops.env.overlay_keys_stable` (static, Pure): overlay key sets are stable and schema-versioned across base/dev/ci/prod

#### OPS-ENV-009 environment overlays directory boundary contract

Tests:
- `ops.env.overlays_dir_no_stray_files` (static, Pure): overlays directory has no stray files and portability matrix is present

### Pillar: inventory

#### OPS-INV-001 inventory completeness contract

Tests:
- `ops.inventory.completeness` (static, Pure): inventory registers all domains and policy files

#### OPS-INV-002 inventory orphan files contract

Tests:
- `ops.inventory.no_orphan_files` (static, Pure): ops files must be mapped through inventory sources

#### OPS-INV-003 inventory duplicate source contract

Tests:
- `ops.inventory.no_duplicate_ssot_sources` (static, Pure): duplicate ssot markdown sources are forbidden

#### OPS-INV-004 inventory authority tier contract

Tests:
- `ops.inventory.authority_tiers_enforced` (static, Pure): authority tier exceptions are structured and expiry-bound

#### OPS-INV-005 inventory control graph contract

Tests:
- `ops.inventory.control_graph_validated` (static, Pure): control graph edges and node mappings are valid and acyclic

#### OPS-INV-006 inventory contract id format contract

Tests:
- `ops.inventory.contract_id_format` (static, Pure): all ops contract ids follow OPS-<PILLAR>-NNN format

#### OPS-INV-007 inventory gates registry contract

Tests:
- `ops.inventory.gates_registry_mapped` (static, Pure): gates registry exists and maps each gate to one action id

#### OPS-INV-008 inventory drills registry contract

Tests:
- `ops.inventory.drills_registry_mapped` (static, Pure): drills registry ids map to runnable observe drill definitions

#### OPS-INV-009 inventory owners registry contract

Tests:
- `ops.inventory.owners_registry_complete` (static, Pure): owners registry exists and includes all ops domain directories

#### OPS-INV-010 inventory schema coverage contract

Tests:
- `ops.inventory.schema_coverage` (static, Pure): inventory schema directory includes required registry schemas

#### OPS-INV-MAP-001 inventory contract gate map coverage contract

Tests:
- `ops.inventory.contract_gate_map.every_contract_mapped` (static, Pure): every contract id is mapped in contract-gate-map

#### OPS-INV-MAP-002 inventory contract gate map gate reference contract

Tests:
- `ops.inventory.contract_gate_map.mapped_gates_exist` (static, Pure): every mapped gate id exists in gates registry

#### OPS-INV-MAP-003 inventory contract gate map command surface contract

Tests:
- `ops.inventory.contract_gate_map.mapped_commands_registered` (static, Pure): every mapped command is registered in ops command surface

#### OPS-INV-MAP-004 inventory contract gate map effects annotation contract

Tests:
- `ops.inventory.contract_gate_map.effects_annotation_matches_mode` (static, Pure): effects annotations match contract test kinds and execution mode

#### OPS-INV-MAP-005 inventory contract gate map orphan gate contract

Tests:
- `ops.inventory.contract_gate_map.no_orphan_gates` (static, Pure): every gate id is referenced by at least one contract mapping

#### OPS-INV-MAP-006 inventory contract gate map orphan contract contract

Tests:
- `ops.inventory.contract_gate_map.no_orphan_contracts` (static, Pure): every contract maps to gate ids or is explicitly static-only

#### OPS-INV-MAP-007 inventory contract gate map static purity contract

Tests:
- `ops.inventory.contract_gate_map.static_only_contracts_are_pure` (static, Pure): static-only mappings are restricted to pure test contracts

#### OPS-INV-MAP-008 inventory contract gate map effect kind contract

Tests:
- `ops.inventory.contract_gate_map.effect_contracts_require_effect_kind` (static, Pure): effect mappings require subprocess or network test kinds and annotations

#### OPS-INV-MAP-009 inventory contract gate map explain coverage contract

Tests:
- `ops.inventory.contract_gate_map.explain_shows_mapped_gates` (static, Pure): contract explain source mappings expose gate ids for non-static contracts

#### OPS-INV-MAP-010 inventory contract gate map canonical order contract

Tests:
- `ops.inventory.contract_gate_map.mapping_sorted_canonical` (static, Pure): contract-gate-map is sorted by contract id and canonical json

#### OPS-INV-PILLARS-001 inventory pillars registry contract

Tests:
- `ops.inventory.pillars.exists_and_validates` (static, Pure): pillars.json exists and validates

#### OPS-INV-PILLARS-002 inventory pillar directory contract

Tests:
- `ops.inventory.pillars.every_pillar_dir_exists` (static, Pure): every declared non-root pillar has a matching ops directory

#### OPS-INV-PILLARS-003 inventory pillar strictness contract

Tests:
- `ops.inventory.pillars.no_extra_pillar_dirs` (static, Pure): ops root has no undeclared pillar directories

### Pillar: k8s

#### OPS-K8S-001 k8s static chart render contract

Tests:
- `ops.k8s.chart_renders_static` (static, Pure): helm chart source includes required files and static render inputs

#### OPS-K8S-002 k8s values schema validation contract

Tests:
- `ops.k8s.values_files_validate_schema` (static, Pure): install-matrix values files exist and are parseable against chart schema surface

#### OPS-K8S-003 k8s install matrix completeness contract

Tests:
- `ops.k8s.install_matrix_complete` (static, Pure): install matrix covers canonical profile set and references existing files

#### OPS-K8S-004 k8s forbidden object policy contract

Tests:
- `ops.k8s.no_forbidden_k8s_objects` (static, Pure): helm templates must not introduce forbidden cluster-scope object kinds

#### OPS-K8S-005 k8s rbac minimalism contract

Tests:
- `ops.k8s.rbac_minimalism` (static, Pure): helm templates must not declare cluster-admin or wildcard rbac grants

#### OPS-K8S-006 k8s pod security and probes contract

Tests:
- `ops.k8s.pod_security_and_probes` (static, Pure): deployment template includes pod security hardening and readiness/liveness probes

#### OPS-K8S-007 k8s rollout safety contract

Tests:
- `ops.k8s.rollout_safety_enforced` (static, Pure): rollout safety contract is valid and rollout template enforces rollout steps

#### OPS-K8S-008 k8s conformance suite contract

Tests:
- `ops.k8s.conformance_suite_runnable` (static, Pure): k8s conformance suite exists and control-plane exposes conformance verb

#### OPS-K8S-009 k8s install matrix generated consistency contract

Tests:
- `ops.k8s.install_matrix_and_generated_consistency` (static, Pure): install matrix and generated k8s artifacts stay aligned and schema-versioned

#### OPS-K8S-010 k8s generated index determinism contract

Tests:
- `ops.k8s.generated_indexes_deterministic_schema_valid` (static, Pure): generated k8s indexes are schema-versioned and deterministic

#### OPS-K8S-E-001 k8s effect helm install contract

Tests:
- `ops.k8s.effect.helm_install_contract_defined` (effect, Subprocess): effect lane requires kind install profile in k8s install matrix

#### OPS-K8S-E-002 k8s effect rollout safety contract

Tests:
- `ops.k8s.effect.rollout_safety_contract_satisfied` (effect, Subprocess): effect lane requires rollout safety contract checks

#### OPS-K8S-E-003 k8s effect endpoint reachability contract

Tests:
- `ops.k8s.effect.service_endpoints_reachable_contract` (effect, Network): effect lane requires non-empty k8s suite coverage for endpoint checks

### Pillar: load

#### OPS-LOAD-001 load scenario schema contract

Tests:
- `ops.load.scenarios_schema_valid` (static, Pure): load scenarios are parseable and include required fields

#### OPS-LOAD-002 load thresholds coverage contract

Tests:
- `ops.load.thresholds_exist_for_each_suite` (static, Pure): every load suite has a matching thresholds file

#### OPS-LOAD-003 load pinned query lock contract

Tests:
- `ops.load.pinned_queries_lock_consistent` (static, Pure): pinned query lock digests match source query payload

#### OPS-LOAD-004 load baseline schema contract

Tests:
- `ops.load.baselines_schema_valid` (static, Pure): load baselines are parseable and contain required fields

#### OPS-LOAD-005 load scenario to slo mapping contract

Tests:
- `ops.load.no_scenario_without_slo_mapping` (static, Pure): smoke/pr load suites must be represented in inventory SLO mappings

#### OPS-LOAD-006 load drift report schema contract

Tests:
- `ops.load.drift_report_generator_schema_valid` (static, Pure): load drift report exists and is schema-valid

#### OPS-LOAD-007 load result schema sample contract

Tests:
- `ops.load.result_schema_validates_sample_output` (static, Pure): load result schema validates generated sample summary envelope

#### OPS-LOAD-008 load cheap survival suite gate contract

Tests:
- `ops.load.cheap_survival_in_minimal_gate_suite` (static, Pure): cheap-only-survival suite is present in minimal gate lanes

#### OPS-LOAD-009 load cold start p99 suite gate contract

Tests:
- `ops.load.cold_start_p99_in_minimal_gate_suite` (static, Pure): cold-start-p99 suite is present in required confidence lanes

#### OPS-LOAD-010 load scenario slo impact mapping contract

Tests:
- `ops.load.every_scenario_has_slo_impact_class` (static, Pure): every load suite maps to a scenario slo impact class entry

#### OPS-LOAD-E-001 load effect k6 execution contract

Tests:
- `ops.load.effect.k6_suite_executes_contract` (effect, Subprocess): effect lane requires at least one k6 load suite definition

#### OPS-LOAD-E-002 load effect thresholds report contract

Tests:
- `ops.load.effect.thresholds_enforced_report_emitted` (effect, Subprocess): effect lane requires thresholds contract and emitted load summary report

#### OPS-LOAD-E-003 load effect baseline comparison contract

Tests:
- `ops.load.effect.baseline_comparison_produced` (effect, Subprocess): effect lane requires emitted load drift comparison report

### Pillar: observe

#### OPS-OBS-001 observability alert rules contract

Tests:
- `ops.observe.alert_rules_exist_parseable` (static, Pure): required alert rule sources exist and are parseable

#### OPS-OBS-002 observability dashboard golden contract

Tests:
- `ops.observe.dashboard_json_parseable_golden_diff` (static, Pure): dashboard json parses and matches golden identity and panel structure

#### OPS-OBS-003 observability telemetry golden profile contract

Tests:
- `ops.observe.telemetry_goldens_required_profiles` (static, Pure): telemetry goldens exist for required profiles and are indexed

#### OPS-OBS-004 observability readiness schema contract

Tests:
- `ops.observe.readiness_schema_valid` (static, Pure): readiness contract is parseable and uses canonical requirement set

#### OPS-OBS-005 observability alert catalog generation contract

Tests:
- `ops.observe.alert_catalog_generated_consistency` (static, Pure): alert catalog is populated and aligned with parsed alert rules

#### OPS-OBS-006 observability slo burn-rate consistency contract

Tests:
- `ops.observe.slo_definitions_burn_rate_consistent` (static, Pure): slo definitions and burn-rate rules remain aligned

#### OPS-OBS-007 observability public surface coverage contract

Tests:
- `ops.observe.public_surface_coverage_matches_rules` (static, Pure): public surface coverage rules include required surfaces

#### OPS-OBS-008 observability telemetry index determinism contract

Tests:
- `ops.observe.telemetry_index_generated_deterministic` (static, Pure): telemetry index is schema-versioned and sorted deterministically

#### OPS-OBS-009 observability drills manifest contract

Tests:
- `ops.observe.drills_manifest_exists_runnable` (static, Pure): drills manifest is populated with runnable drill definitions

#### OPS-OBS-010 observability overload behavior contract

Tests:
- `ops.observe.overload_behavior_contract_enforced` (static, Pure): overload behavior contract exists and maps to load suite coverage

#### OPS-OBS-E-001 observe effect metrics scrape contract

Tests:
- `ops.observe.effect.scrape_metrics_contract` (effect, Network): effect lane requires non-empty metrics scrape contract

#### OPS-OBS-E-002 observe effect trace structure contract

Tests:
- `ops.observe.effect.trace_structure_contract` (effect, Network): effect lane requires trace structure golden contract

#### OPS-OBS-E-003 observe effect alerts load contract

Tests:
- `ops.observe.effect.alerts_load_contract` (effect, Network): effect lane requires parseable alert rule inputs

#### OPS-RPT-001 report schema ssot contract

Tests:
- `ops.report.schema_is_ssot` (static, Pure): report schema is parseable and mirrored under ops/schema/report

#### OPS-RPT-002 report generated payload contract

Tests:
- `ops.report.generated_reports_schema_valid` (static, Pure): generated report payloads are parseable and include schema_version

#### OPS-RPT-003 report evidence levels contract

Tests:
- `ops.report.evidence_levels_complete` (static, Pure): evidence levels include minimal standard and forensic

#### OPS-RPT-004 report diff structure contract

Tests:
- `ops.report.diff_contract_exists` (static, Pure): generated report diff includes base target and change set

#### OPS-RPT-005 report readiness score determinism contract

Tests:
- `ops.report.readiness_score_deterministic` (static, Pure): readiness score report is schema-versioned and uses canonical input keys

#### OPS-RPT-006 report release evidence bundle contract

Tests:
- `ops.report.release_evidence_bundle_schema_valid` (static, Pure): release evidence bundle is parseable and references existing artifacts

#### OPS-RPT-007 report historical comparison contract

Tests:
- `ops.report.historical_comparison_schema_valid` (static, Pure): historical comparison report includes schema and readiness trend fields

#### OPS-RPT-008 report unified example contract

Tests:
- `ops.report.unified_report_example_schema_valid` (static, Pure): unified report example includes required schema and summary sections

#### OPS-RPT-009 report canonical json output contract

Tests:
- `ops.report.outputs_canonical_json` (static, Pure): report outputs are canonical pretty json with deterministic key ordering

#### OPS-RPT-010 report lane aggregation contract

Tests:
- `ops.report.lane_reports_aggregated_in_unified_report` (static, Pure): unified report summary totals are derived from lane report statuses

### Pillar: root-surface

#### OPS-000 ops directory contract

Tests:
- `ops.dir.allowed_root_files` (static, Pure): ops root allows only contract/readme root files
- `ops.dir.forbid_extra_markdown_root` (static, Pure): ops root forbids extra markdown
- `ops.dir.allow_only_known_domain_dirs` (static, Pure): ops root allows only canonical domain directories
- `ops.dir.forbid_extra_markdown_recursive` (static, Pure): ops forbids recursive markdown outside approved surface

#### OPS-001 ops generated lifecycle contract

Tests:
- `ops.generated.runtime.allowed_files` (static, Pure): ops/_generated allows only runtime artifact formats
- `ops.generated.example.allowed_files` (static, Pure): ops/_generated.example allows only committed artifact formats
- `ops.generated.runtime.no_example_files` (static, Pure): ops/_generated forbids example artifacts

#### OPS-002 ops required domain files contract

Tests:
- `ops.domain.required_contract_and_readme` (static, Pure): each ops domain includes README.md and CONTRACT.md
- `ops.domain.forbid_legacy_docs` (static, Pure): legacy domain INDEX/OWNER/REQUIRED markdown files are forbidden

#### OPS-003 ops markdown budget contract

Tests:
- `ops.markdown_budget.readme` (static, Pure): README markdown files stay within line budget
- `ops.markdown_budget.contract` (static, Pure): CONTRACT markdown files stay within line budget

#### OPS-004 ops docs ssot boundary contract

Tests:
- `ops.docs.readme_ssot_boundary` (static, Pure): ops root readme remains navigation-only and references docs/operations

#### OPS-005 ops contract document generation contract

Tests:
- `ops.contract_doc.generated_match` (static, Pure): ops CONTRACT.md matches generated output from contract registry

#### OPS-DOCS-001 operations docs policy linkage contract

Tests:
- `ops.docs.policy_keyword_requires_contract_id` (static, Pure): operations docs with policy keywords must reference OPS contract ids
- `ops.docs.index_crosslinks_contracts` (static, Pure): operations index must state docs/contracts boundary and include OPS references

#### OPS-ROOT-001 ops root allowed surface contract

Tests:
- `ops.root.allowed_surface` (static, Pure): ops root contains only canonical files and domain directories

#### OPS-ROOT-002 ops root markdown contract

Tests:
- `ops.root.forbid_extra_markdown` (static, Pure): ops root forbids markdown files other than README.md and CONTRACT.md

#### OPS-ROOT-003 ops no shell scripts contract

Tests:
- `ops.root.no_shell_script_files` (static, Pure): ops tree contains no shell script files or bash shebangs

#### OPS-ROOT-004 ops max directory depth contract

Tests:
- `ops.root.max_directory_depth` (static, Pure): ops file paths remain within configured depth budget

#### OPS-ROOT-005 ops filename policy contract

Tests:
- `ops.root.filename_policy` (static, Pure): ops filenames follow stable lowercase policy with explicit allowlist exceptions

#### OPS-ROOT-006 ops generated gitignore policy contract

Tests:
- `ops.root.generated_gitignore_policy` (static, Pure): ops/_generated is gitignored with explicit .gitkeep exception

#### OPS-ROOT-007 ops generated example secret guard contract

Tests:
- `ops.root.generated_example_secret_guard` (static, Pure): ops/_generated.example is secret-free and json payloads are parseable

#### OPS-ROOT-008 ops placeholder directory contract

Tests:
- `ops.root.placeholder_dirs_allowlist` (static, Pure): ops placeholder directories are explicitly allowlisted

#### OPS-ROOT-009 ops policy inventory coverage contract

Tests:
- `ops.root.policy_files_inventory_coverage` (static, Pure): ops policy/config files are covered by inventory sources

#### OPS-ROOT-010 ops deleted doc name guard contract

Tests:
- `ops.root.forbid_deleted_doc_names` (static, Pure): forbidden legacy ops markdown names must not be reintroduced

#### OPS-ROOT-SURFACE-001 ops root command surface required commands contract

Tests:
- `ops.root_surface.required_commands_exist` (static, Pure): required ops command verbs exist in command surface registry

#### OPS-ROOT-SURFACE-002 ops root command surface no hidden commands contract

Tests:
- `ops.root_surface.no_hidden_commands` (static, Pure): listed commands and action dispatch entries must match exactly

#### OPS-ROOT-SURFACE-003 ops root command surface deterministic ordering contract

Tests:
- `ops.root_surface.surface_ordering_deterministic` (static, Pure): command surface list is sorted deterministically

#### OPS-ROOT-SURFACE-004 ops root command surface effects declaration contract

Tests:
- `ops.root_surface.commands_declare_effects` (static, Pure): mapped commands declare effects_required annotations

#### OPS-ROOT-SURFACE-005 ops root command surface pillar grouping contract

Tests:
- `ops.root_surface.commands_grouped_by_pillar` (static, Pure): ops command actions use approved pillar-style domain groups

#### OPS-ROOT-SURFACE-006 ops root command surface ad-hoc group guard contract

Tests:
- `ops.root_surface.forbid_adhoc_command_groups` (static, Pure): ops command actions must not use ad-hoc misc/util group names

### Pillar: schema

#### OPS-SCHEMA-001 schema parseability contract

Tests:
- `ops.schema.parseable_documents` (static, Pure): ops json/yaml policy documents are parseable

#### OPS-SCHEMA-002 schema index completeness contract

Tests:
- `ops.schema.index_complete` (static, Pure): generated schema index covers all schema sources

#### OPS-SCHEMA-003 schema naming contract

Tests:
- `ops.schema.no_unversioned` (static, Pure): schema sources use stable .schema.json naming

#### OPS-SCHEMA-004 schema budget contract

Tests:
- `ops.schema.budget_policy` (static, Pure): schema count stays within per-domain budgets

#### OPS-SCHEMA-005 schema evolution lock contract

Tests:
- `ops.schema.evolution_lock` (static, Pure): compatibility lock tracks schema evolution targets

#### OPS-SCHEMA-006 schema id consistency contract

Tests:
- `ops.schema.id_and_naming_consistency` (static, Pure): schema files define stable $id values aligned with file names

#### OPS-SCHEMA-007 schema example validation contract

Tests:
- `ops.schema.examples_validate_required_fields` (static, Pure): schema examples satisfy required field coverage from compatibility lock

#### OPS-SCHEMA-008 schema intent uniqueness contract

Tests:
- `ops.schema.forbid_duplicate_intent` (static, Pure): schema ids and titles are unique to avoid duplicated intent

#### OPS-SCHEMA-009 schema canonical formatting contract

Tests:
- `ops.schema.canonical_json_formatting` (static, Pure): generated schema artifacts use canonical pretty json formatting

#### OPS-SCHEMA-010 schema example coverage contract

Tests:
- `ops.schema.example_coverage` (static, Pure): schema compatibility targets declare existing example fixtures

### Pillar: stack

#### OPS-STACK-001 stack toml profile contract

Tests:
- `ops.stack.stack_toml_parseable_complete` (static, Pure): stack.toml parses and includes canonical ci kind local profiles

#### OPS-STACK-002 stack service dependency contract

Tests:
- `ops.stack.service_dependency_contract_valid` (static, Pure): service dependency contract entries are parseable and resolve to files

#### OPS-STACK-003 stack version manifest contract

Tests:
- `ops.stack.versions_manifest_schema_valid` (static, Pure): version manifest is parseable and image refs are digest pinned

#### OPS-STACK-004 stack dependency graph contract

Tests:
- `ops.stack.dependency_graph_generated_acyclic` (static, Pure): dependency graph is parseable and references real cluster/components

#### OPS-STACK-005 stack kind profile consistency contract

Tests:
- `ops.stack.kind_profiles_consistent` (static, Pure): dev perf and small kind profiles exist and reference valid cluster configs

#### OPS-STACK-006 stack ports inventory consistency contract

Tests:
- `ops.stack.ports_inventory_matches_stack` (static, Pure): ports inventory endpoints are unique and aligned with stack components

#### OPS-STACK-007 stack health report generator contract

Tests:
- `ops.stack.health_report_generator_contract` (static, Pure): health report sample has schema envelope and stack generator provenance

#### OPS-STACK-008 stack command surface contract

Tests:
- `ops.stack.stack_commands_registered` (static, Pure): stack command surface snapshot contains up and down verbs

#### OPS-STACK-009 stack offline profile policy contract

Tests:
- `ops.stack.offline_profile_policy` (static, Pure): offline claims require offline or airgap profile coverage

#### OPS-STACK-E-001 stack effect kind cluster contract

Tests:
- `ops.stack.effect.kind_cluster_up_profile_dev` (effect, Subprocess): effect lane requires kind dev cluster contract inputs

#### OPS-STACK-E-002 stack effect component rollout contract

Tests:
- `ops.stack.effect.core_components_present` (effect, Subprocess): effect lane requires core stack component manifests

#### OPS-STACK-E-003 stack effect ports inventory contract

Tests:
- `ops.stack.effect.ports_inventory_mapped` (effect, Subprocess): effect lane requires stack ports inventory contract sample

#### OPS-STACK-E-004 stack effect health report contract

Tests:
- `ops.stack.effect.health_report_generated` (effect, Subprocess): effect lane requires stack health report contract sample

## Rule

- Contract ID or test ID missing from this document means it does not exist.
