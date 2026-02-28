# Configs Contract

`bijux dev atlas contracts configs` is the primary evidence for the configs contract surface. The authoritative registry is `configs/configs.contracts.json`.

| Contract | Severity | Type | Enforced By |
| --- | --- | --- | --- |
| `CFG-001` | `blocker` | `filelayout` | `bijux dev atlas contracts configs` / `configs.root.only_root_docs` |
| `CFG-002` | `must` | `filelayout` | `bijux dev atlas contracts configs` / `configs.layout.depth_budget` |
| `CFG-003` | `blocker` | `drift` | `bijux dev atlas contracts configs` / `configs.registry.no_overlap` |
| `CFG-004` | `blocker` | `schema` | `bijux dev atlas contracts configs` / `configs.schemas.file_coverage` |
| `CFG-005` | `must` | `schema` | `bijux dev atlas contracts configs` / `configs.registry.schema_owner` |
| `CFG-006` | `must` | `supplychain` | `bijux dev atlas contracts configs` / `configs.lockfiles.required_pairs` |
| `CFG-007` | `must` | `static` | `bijux dev atlas contracts configs` / `configs.registry.visibility_classification` |
| `CFG-008` | `must` | `supplychain` | `bijux dev atlas contracts configs` / `configs.lockfiles.required_pairs` |
| `CFG-009` | `blocker` | `drift` | `bijux dev atlas contracts configs` / `configs.owners.group_alignment` |
| `CFG-010` | `blocker` | `drift` | `bijux dev atlas contracts configs` / `configs.consumers.file_coverage` |
| `CFG-011` | `must` | `filelayout` | `bijux dev atlas contracts configs` / `configs.registry.group_budget` |
| `CFG-012` | `blocker` | `drift` | `bijux dev atlas contracts configs` / `configs.registry.no_orphans` |
| `CFG-013` | `must` | `drift` | `bijux dev atlas contracts configs` / `configs.registry.no_dead_entries` |
| `CFG-014` | `must` | `schema` | `bijux dev atlas contracts configs` / `configs.generated_index.committed_match` |
| `CFG-015` | `should` | `supplychain` | `bijux dev atlas contracts configs` / `configs.consumers.group_alignment` |
| `CFG-016` | `should` | `supplychain` | `bijux dev atlas contracts configs` / `configs.parse.json` |
| `CFG-017` | `must` | `filelayout` | `bijux dev atlas contracts configs` / `configs.registry.complete_surface` |
| `CFG-018` | `must` | `drift` | `bijux dev atlas contracts configs` / `configs.generated_index.deterministic` |
| `CFG-019` | `must` | `drift` | `bijux dev atlas contracts configs` / `configs.registry.lifecycle` |
| `CFG-020` | `must` | `drift` | `bijux dev atlas contracts configs` / `configs.registry.tool_entrypoints` |
| `CFG-021` | `must` | `schema` | `bijux dev atlas contracts configs` / `configs.registry.schema_owner` |
| `CFG-022` | `blocker` | `drift` | `bijux dev atlas contracts configs` / `configs.registry.no_undocumented_files` |
| `CFG-023` | `must` | `filelayout` | `bijux dev atlas contracts configs` / `configs.naming.internal_surface` |
| `CFG-024` | `must` | `filelayout` | `bijux dev atlas contracts configs` / `configs.generated.authored_boundary` |
| `CFG-025` | `must` | `schema` | `bijux dev atlas contracts configs` / `configs.parse.json` |
| `CFG-026` | `must` | `schema` | `bijux dev atlas contracts configs` / `configs.parse.yaml` |
| `CFG-027` | `must` | `schema` | `bijux dev atlas contracts configs` / `configs.parse.toml` |
| `CFG-028` | `must` | `drift` | `bijux dev atlas contracts configs` / `configs.text.hygiene` |
| `CFG-029` | `blocker` | `filelayout` | `bijux dev atlas contracts configs` / `configs.docs.no_nested_markdown` |
| `CFG-030` | `must` | `filelayout` | `bijux dev atlas contracts configs` / `configs.docs.tooling_surface` |
| `CFG-031` | `blocker` | `drift` | `bijux dev atlas contracts configs` / `configs.owners.group_alignment` |
| `CFG-032` | `blocker` | `drift` | `bijux dev atlas contracts configs` / `configs.contracts.no_policy_theater` |
| `CFG-033` | `blocker` | `drift` | `bijux dev atlas contracts configs` / `configs.registry.owner_complete` |
| `CFG-034` | `must` | `schema` | `bijux dev atlas contracts configs` / `configs.schema.coverage` |
| `CFG-035` | `must` | `filelayout` | `bijux dev atlas contracts configs` / `configs.registry.group_depth_budget` |
| `CFG-036` | `must` | `drift` | `bijux dev atlas contracts configs` / `configs.json.canonical_root_surface` |
| `CFG-037` | `must` | `drift` | `bijux dev atlas contracts configs` / `configs.schema.index_committed_match` |
| `CFG-038` | `must` | `schema` | `bijux dev atlas contracts configs` / `configs.schema.no_orphan_inputs` |
| `CFG-039` | `must` | `schema` | `bijux dev atlas contracts configs` / `configs.schema.versioning_policy` |
