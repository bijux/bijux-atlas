# Generated Committed Mirror Policy

`ops/_generated.example/` is a committed mirror/output compatibility area during the migration window.

Authoritative lifecycle contract: `ops/GENERATED_LIFECYCLE.md`.

Rules:
- Generate primary outputs under `ops/_generated/`.
- Only explicit update commands may write `ops/_generated.example/`.
- Lifecycle class for this directory is `curated_evidence`.
- Every committed file in `ops/_generated.example/` must be declared in `ops/_generated.example/ALLOWLIST.json`.
- `ops/_generated.example/inventory-index.json` is the generated inventory checksum index used for drift comparisons.
- `ops/_generated.example/control-plane.snapshot.md` is the control-plane snapshot used for drift checks.
- `ops/_generated.example/control-plane-drift-report.json` is the control-plane contract drift status artifact.
- `ops/_generated.example/control-plane-surface-list.json` is the control-plane command-surface and crate-alignment report.
- `ops/_generated.example/docs-drift-report.json` is the docs governance drift report artifact.
- `ops/_generated.example/file-usage-report.json` is the generated usage graph and orphan detector for ops JSON/YAML/TOML artifacts.
- `ops/_generated.example/ops-ledger.json` is the generated ledger of all ops files with proof-of-necessity metadata.
- `ops/_generated.example/ops-ledger.md` is the human-readable summary of ops-ledger classifications.
- `ops/_generated.example/orphan-files-report.json` is the generated orphan artifact report derived from the ops ledger.
- `ops/_generated.example/registry-graph.json` is the generated contracts-map to schema/artifact relationship graph.
- `ops/_generated.example/docs-links-report.json` is the generated docs-to-ops path reference report.
- `ops/_generated.example/schema-coverage-report.json` is the generated schema/contract coverage report for ops data files.
- `ops/_generated.example/generator-coverage-report.json` is the generated report for generated artifacts missing generator metadata.
- `ops/_generated.example/placeholder-dirs-report.json` is the generated placeholder directory inventory and drift detector.
- `ops/_generated.example/registry-drift-report.json` is the generated inventory registry drift summary.
- `ops/_generated.example/contract-coverage-report.json` is the generated contract coverage summary.
- `ops/_generated.example/schema-drift-report.json` is the generated schema governance drift summary.
- `ops/_generated.example/fixture-drift-report.json` is the generated fixture governance drift summary.
- Binary artifacts are forbidden in this directory.
- Every committed JSON artifact in this directory must include `schema_version`.
- Every committed JSON artifact in this directory must include `generated_by`.

## Generator Commands

- `ops/_generated.example/ops-index.json`: `bijux dev atlas report build-index --write-example`
- `ops/_generated.example/ops-evidence-bundle.json`: `bijux dev atlas report build-evidence --write-example`
- `ops/_generated.example/scorecard.json`: `bijux dev atlas report build-scorecard --write-example`
- `ops/_generated.example/pins.index.example.json`: `bijux dev atlas inventory pins index --write-example`
- `ops/_generated.example/inventory-index.json`: `bijux dev atlas inventory index --write-example`
- `ops/_generated.example/control-plane.snapshot.md`: `bijux dev atlas ops control-plane snapshot --write-example`
- `ops/_generated.example/control-plane-drift-report.json`: `bijux dev atlas ops control-plane drift-report --write-example`
- `ops/_generated.example/control-plane-surface-list.json`: `bijux dev atlas ops control-plane surface-list --write-example`
- `ops/_generated.example/docs-drift-report.json`: `bijux dev atlas docs drift --write-example`
- `ops/_generated.example/file-usage-report.json`: `bijux dev atlas ops inventory file-usage --write-example`
- `ops/_generated.example/ops-ledger.json`: `bijux dev atlas ops inventory ledger --write-example`
- `ops/_generated.example/ops-ledger.md`: `bijux dev atlas ops inventory ledger --write-example --format md`
- `ops/_generated.example/orphan-files-report.json`: `bijux dev atlas ops inventory orphan-report --write-example`
- `ops/_generated.example/registry-graph.json`: `bijux dev atlas ops inventory registry-graph --write-example`
- `ops/_generated.example/docs-links-report.json`: `bijux dev atlas ops docs links-report --write-example`
- `ops/_generated.example/schema-coverage-report.json`: `bijux dev atlas ops schema coverage-report --write-example`
- `ops/_generated.example/generator-coverage-report.json`: `bijux dev atlas ops generated coverage-report --write-example`
- `ops/_generated.example/placeholder-dirs-report.json`: `bijux dev atlas ops inventory placeholder-dirs-report --write-example`
- `ops/_generated.example/registry-drift-report.json`: `bijux dev atlas ops inventory registry-drift-report --write-example`
- `ops/_generated.example/fixture-drift-report.json`: `bijux dev atlas ops fixtures drift --write-example`

## Mirrored Artifacts

- `ops/_generated.example/ALLOWLIST.json`: machine-checkable whitelist for committed artifacts in this directory.
- `ops/_generated.example/ops-index.json`: canonical generated index of ops domains and reporting outputs.
- `ops/_generated.example/ops-evidence-bundle.json`: canonical generated evidence envelope with hashes and gate status.
- `ops/_generated.example/scorecard.json`: generated readiness score summary.
- `ops/_generated.example/pins.index.example.json`: generated pins-index example contract.
- `ops/_generated.example/inventory-index.json`: generated checksum index for inventory SSOT files.
- `ops/_generated.example/control-plane.snapshot.md`: generated control-plane snapshot for drift enforcement.
- `ops/_generated.example/control-plane-drift-report.json`: generated control-plane drift report covering policy-only doc contract and cargo-metadata alignment.
- `ops/_generated.example/control-plane-surface-list.json`: generated control-plane surface list report aligned with cargo metadata and command ownership.
- `ops/_generated.example/docs-drift-report.json`: generated docs governance drift report.
- `ops/_generated.example/file-usage-report.json`: generated file usage graph with orphan classifications across ops data artifacts.
- `ops/_generated.example/ops-ledger.json`: generated authoritative ops file ledger with classification and necessity reasons.
- `ops/_generated.example/ops-ledger.md`: generated summary view of the ledger for review diffs.
- `ops/_generated.example/orphan-files-report.json`: generated orphan artifact report used for merge blocking.
- `ops/_generated.example/registry-graph.json`: generated inventory contracts-map dependency graph.
- `ops/_generated.example/docs-links-report.json`: generated docs markdown references to ops artifact paths.
- `ops/_generated.example/schema-coverage-report.json`: generated schema and contract coverage report for ops artifacts.
- `ops/_generated.example/generator-coverage-report.json`: generated coverage report for generated artifact metadata.
- `ops/_generated.example/placeholder-dirs-report.json`: generated placeholder directory report aligned to the inventory allowlist.
- `ops/_generated.example/registry-drift-report.json`: generated inventory registry drift report for missing or extra inventory artifacts.
- `ops/_generated.example/contract-coverage-report.json`: generated contract coverage report for domain contract invariants and check linkage.
- `ops/_generated.example/schema-drift-report.json`: generated schema drift report for index, compatibility lock, and allowlist governance.
- `ops/_generated.example/fixture-drift-report.json`: generated fixture drift report for fixture inventory, allowlist, and binary policy governance.
