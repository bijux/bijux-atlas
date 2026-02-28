# Documentation Inventory And Triage Report

Owner: `docs-governance`  
Status: `active`  
Date: `2026-02-28`

## Scope Covered

This report records completion for controls `41` through `60`.

## Inventory Coverage

- Full inventory table: `docs/governance/docs-inventory.csv`.
- Inventory fields: `path`, `title`, `type`, `audience`, `disposition`, `reason`.
- Total inventoried markdown pages: `479`.
- Disposition tags applied to every page: `KEEP`, `MERGE`, `DELETE`, `DRAFT`.

## Duplicate Topic Findings

High-value duplicate clusters requiring consolidation:

- `compatibility.md` across API, contracts, and observability.
- `errors.md` across API, contracts, and reference.
- `glossary` content split across root and `operations/slo`.
- `make-targets.md` split across generated, development, and reference.
- Governance duplication between `_style/` and `governance/` for review/removal/style guides.
- Duplicate boundary content: `architecture/no-layer-fixups.md` and `architecture/layering/no-layer-fixups.md`.

## Dead-End Page Findings

- Pages not linked from section entrypoints: `431`.
- Source list: `docs/governance/docs-dead-end-pages.txt`.

## Index Chain Findings

Index-heavy chain pages identified:

- `docs/science/index.md`
- `docs/security/index.md`

These pages link only to other index pages and should be flattened into canonical section entrypoints.

## Obsolete Surface Findings

- Broken internal links detected: `22`.
- Source list: `docs/governance/docs-broken-links.csv`.
- Notable stale references include missing generated contract files and old uppercase path links such as `../start-here.md`.

## Policy Placement Findings

- Policy-like pages outside governance: `48`.
- Source list: `docs/governance/docs-policy-pages-outside-governance.txt`.
- Decision: policy pages should converge into `docs/governance/` unless they are strict runtime runbooks.

## Reference Narrative Findings

Reference pages are mostly factual, but overlap exists where operational and reference surfaces duplicate command, drill, and gate tables.

- Overlap list: `docs/governance/docs-operations-reference-overlap.txt`.
- Decision: keep factual tables in `docs/reference/` and de-duplicate parallel copies under `docs/operations/reference/`.

## Runbook Structure Findings

Runbooks missing explicit prerequisites and failure-mode sections: `17` files under `docs/operations/runbooks/`.

- Immediate examples: `store-outage.md`, `traffic-spike.md`, `rollback-playbook.md`, `dataset-corruption.md`.
- Decision: either add required sections or merge/delete non-actionable runbooks.

## Historical And Legacy Content Findings

Historical-transition content is still present in active paths, including migration-only pages.

- Key examples: `docs/architecture/layout-shell-migration.md`, `docs/architecture/removed-compatibility-paths.md`, `docs/development/task-runner-removal-map.md`.
- Decision: move historical-only material to `_drafts/` with expiry or delete after canonical content absorbs required facts.

## Scripts And Legacy Tooling Findings

Legacy tooling references remain in development docs and architecture notes.

- High-risk examples: `docs/development/scripts-graph.md`, `docs/development/task-runner-removal-map.md`, `docs/development/tooling/scripts-compat-policy.md`.
- Decision: remove script-era narratives that no longer represent active workflows.

## Map Document Findings

Map-like pages detected: `10`.

- Source list includes architecture, operations, development, and root map documents.
- Decision: keep only maps that capture enduring boundaries; remove repo-structure mirrors.

## Concept Registry Findings

Concept registry surfaces:

- `docs/_generated/concept-governance/metadata/registry.json`
- `docs/_generated/concept-registry.md`
- `docs/_style/CONCEPT_REGISTRY.md`

Decision: keep generated registry outputs; merge or remove `_style` authored duplicate.

## Topic/Search Output Findings

Generated discovery outputs detected:

- `docs/_generated/topic-index.md`
- `docs/_generated/topic-index.json`
- `docs/_generated/search-index.json`

Decision: keep as generated-only artifacts, not authored docs.

## Uppercase Index Findings

- Uppercase `INDEX.md` pages still present: `44`.
- Source list: `docs/governance/docs-uppercase-index-pages.txt`.
- Decision: convert retained sections to `index.md` and remove superseded sections.

## `docs/root/` Decision

- Current state: `docs/root/` duplicates root-level documentation intent.
- Decision: delete `docs/root/` after unique content is merged into canonical root and section pages.

## Operations/Reference Overlap Decision

- Overlap hotspot: `docs/operations/reference/` duplicates command and schema-style reference pages.
- Decision: migrate factual reference pages into `docs/reference/`; keep operations pages strictly procedural.

## Development Tooling Sprawl Decision

- Tooling pages under `docs/development/tooling/`: `15`.
- Source list: `docs/governance/docs-development-tooling-pages.txt`.
- Decision: consolidate into a smaller contributor-focused set and remove historical transition notes.

## ADR Location Decision

- Current ADR location: `docs/adrs/`.
- Decision: keep ADRs and move to `docs/governance/adrs/` as the canonical policy-decision location.

## Single-Link Support Page Findings

Pages with only one inbound link (candidate support-only pages): `192`.

- Source list: `docs/governance/docs-single-inbound-pages.txt`.
- Decision: evaluate single-link pages for merge/delete, especially where both parent and child are low-value maps or historical notes.
