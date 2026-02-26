# Gates Reference

- Owner: `bijux-atlas-operations`
- Tier: `generated`
- Audience: `operators`
- Source-of-truth: `ops/inventory/gates.json`

## Gates

| Gate ID | Category | Action ID | Description |
| --- | --- | --- | --- |
| `ops.doctor` | `root` | `ops.root.doctor` | baseline inventory and contract health |
| `ops.validate` | `root` | `ops.root.check` | full contract validation lane |
| `ops.gate.directory-completeness` | `governance` | `ops.root.directory-budgets-check` | directory completeness and budget checks |
| `ops.gate.schema-validation` | `schema` | `ops.root.schema-check` | schema validation and drift detection |
| `ops.gate.schema-locality` | `schema` | `ops.root.schema-check` | schema references must start with ops/schema/ |
| `ops.gate.no-external-schema-files` | `schema` | `ops.root.schema-check` | schema files are allowed only under ops/schema/** |
| `ops.gate.pin-drift` | `inventory` | `ops.pins.check` | pin drift and coverage checks |
| `ops.gate.stack-reproducibility` | `stack` | `ops.stack.validate` | stack reproducibility and profile validation |
| `ops.gate.k8s-determinism` | `k8s` | `ops.k8s.check` | k8s render/install determinism checks |
| `ops.gate.observe-coverage` | `observe` | `ops.observe.validate` | observability coverage and readiness checks |
| `ops.gate.observe-naming` | `governance` | `ops.root.naming-check` | forbid legacy obs domain keys and ids; only observe is allowed after 2026-02-25 |
| `ops.gate.inventory-registry-role` | `inventory` | `ops.root.contracts-check` | inventory registry files must declare role as authored or generated |
| `ops.gate.inventory-registry-uniqueness` | `inventory` | `ops.root.contracts-check` | forbid duplicate semantic registry files for the same inventory contract domain |
| `ops.gate.inventory-registry-ordering` | `inventory` | `ops.root.contracts-check` | generated inventory registries must use deterministic stable ordering |
| `ops.gate.inventory-schema-coverage` | `inventory` | `ops.root.schema-check` | every inventory artifact must reference an ops/schema/inventory schema |
| `ops.gate.inventory-schema-index-coverage` | `inventory` | `ops.root.schema-check` | schema index must include all ops/schema/inventory schemas |
| `ops.gate.inventory-owner-fragments-live` | `inventory` | `ops.root.contracts-check` | owner-docs.fragments.json entries must reference live owner docs |
| `ops.gate.inventory-tool-registry-uniqueness` | `inventory` | `ops.root.contracts-check` | tools.toml is the only tool probe registry; registry.toml is check catalog only |
| `ops.gate.inventory-pin-format` | `inventory` | `ops.pins.check` | toolchain and release pins must conform to inventory pin format rules |
| `ops.gate.canonical-directory-names` | `governance` | `ops.root.naming-check` | enforce canonical directory names; contracts/ is allowed and contract/ is forbidden |
| `ops.gate.control-plane-drift` | `docs` | `ops.root.contracts-check` | ops/CONTROL_PLANE.md must match control-plane snapshot contract or avoid hardcoded crate lists |
| `ops.gate.no-stale-locked-claims` | `docs` | `ops.root.contracts-check` | non-generated contract docs must not contain stale locked list claims |
| `ops.gate.dataset-lifecycle` | `datasets` | `ops.datasets.validate` | dataset lifecycle and manifest checks |
| `ops.gate.unified-readiness` | `report` | `ops.observe.report` | unified readiness report publication gate |
| `ops.gate.ssot` | `governance` | `ops.root.check` | ssot invariants: no duplicate authored truth, no obs domain, and no schemas outside ops/schema |
| `ops.gate.validate` | `governance` | `ops.root.check` | validate all json/yaml/toml artifacts and schema contracts |
| `ops.gate.structure` | `governance` | `ops.root.contracts-check` | enforce required files contracts and placeholder directory declarations |
| `ops.gate.docs` | `docs` | `ops.root.contracts-check` | block orphan docs, broken links, and stale contract lists |
| `ops.gate.docs-paths` | `docs` | `ops.root.contracts-check` | docs markdown path links must resolve to existing targets |
| `ops.gate.docs-budget` | `docs` | `ops.root.contracts-check` | docs markdown directory counts must stay within canonical budget caps |
| `ops.gate.generated` | `generated` | `ops.gen.check` | require generated outputs to declare generated_by and schema_version |
| `ops.gate.evidence` | `report` | `ops.root.contracts-check` | enforce curated evidence allowlist and bundle validity |
| `ops.gate.fixtures` | `datasets` | `ops.datasets.validate` | enforce fixture allowlists, locks, version hashes, and inventory drift checks |
| `ops.gate.naming` | `governance` | `ops.root.naming-check` | enforce canonical naming and forbid legacy aliases |
| `ops.gate.inventory` | `inventory` | `ops.root.contracts-check` | enforce deterministic inventory registries and prevent semantic duplicates |
| `ops.gate.schema` | `schema` | `ops.root.schema-check` | enforce schema index, compatibility lock, reference policy, and schema budget |
| `ops.gate.ledger` | `governance` | `ops.root.contracts-check` | enforce ops ledger coverage, necessity reasons, and reference integrity |
| `ops.gate.orphan-report` | `governance` | `ops.root.contracts-check` | orphan-files-report and file-usage-report must match live repository state |
| `ops.gate.binary-policy` | `governance` | `ops.root.contracts-check` | forbid binary or unsupported artifact formats outside ops/**/assets |
