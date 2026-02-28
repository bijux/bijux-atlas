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
