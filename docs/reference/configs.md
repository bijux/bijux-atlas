# Configs Reference

- Owner: `bijux-atlas-operations`
- Tier: `generated`
- Audience: `operators`
- Source-of-truth: `configs/inventory/consumers.json`

## Config Groups

| Group | Consumers |
| --- | --- |
| `ci` | `workflow routing and lane contracts` |
| `cli` | `bijux dev atlas help, bijux dev atlas doctor` |
| `contracts` | `bijux-dev-atlas configs validate, ci schema checks` |
| `coverage` | `coverage policy checks` |
| `docs` | `docs-only workflow, docs-audit workflow` |
| `gates` | `ci lane routing` |
| `inventory` | `bijux-dev-atlas configs inventory, repo governance checks` |
| `layout` | `repository layout checks` |
| `make` | `make governance checks` |
| `meta` | `ownership validation` |
| `nextest` | `cargo nextest test lanes` |
| `openapi` | `openapi snapshot checks` |
| `ops` | `ops validation and conformance` |
| `perf` | `performance threshold checks` |
| `policy` | `policy lint checks` |
| `product` | `artifact manifest checks` |
| `repo` | `repo doctor and surface checks` |
| `reports` | `bijux dev atlas artifacts report inventory, bijux dev atlas artifacts report validate` |
| `rust` | `cargo fmt and clippy` |
| `schema` | `configs schema validation` |
| `security` | `cargo deny and audit` |
| `shellcheck` | `shell lint policy checks` |
| `slo` | `slo contract checks` |
