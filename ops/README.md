# Ops

`ops/` is the repository-owned source of truth for Atlas operational inputs, inventories, schemas, fixtures, and generated examples.

- Intent: keep `ops/` as operational data, schemas, inventories, fixtures, and generated evidence examples.
- Machine validation entrypoint: `bijux-dev-atlas ops validate --format json`.
- Focused execution entrypoints: `bijux-dev-atlas ops profiles ...`, `bijux-dev-atlas ops render ...`, `bijux-dev-atlas ops install ...`, and `bijux-dev-atlas ops stack ...`.
- Human walkthroughs and procedures live in `docs/operations/`.

## Root Docs

- `ops/README.md`: what `ops/` is for and what does not belong here.
- `ops/CONTRACT.md`: durable rules, authorities, and evidence expectations.
- `ops/INDEX.md`: canonical directory map for the live ops surface.
- `ops/ERRORS.md`: root-level error vocabulary for ops validation and repo-law failures.
- `ops/SSOT.md`: markdown policy for `ops/`.

## Design Rules

- Path should tell you whether a file is authored truth, schema, fixture, generated example, or release evidence.
- Inventories under `ops/inventory/` describe operational authorities; they do not replace validation output.
- Generated examples under `ops/_generated.example/` are illustrative evidence mirrors, not authored truth.
- Runtime effect commands require explicit opt-in flags; static inventory and schema checks do not.
- Markdown is intentionally tiny. Narrative and policy prose is limited to the five root docs; deeper `ops/` paths should stay machine-readable.

## Release And Runbook Scope

- Minimum release evidence lives in data, not prose: `ops/inventory/contracts-map.json`, `ops/inventory/authority-index.json`, `ops/load/suites/suites.json`, `ops/observe/drills.json`, and `ops/report/generated/readiness-score.json`.
- Runbook generation is driven by data authorities: `ops/inventory/control-graph.json`, `ops/k8s/install-matrix.json`, `ops/stack/profile-intent.json`, and `ops/inventory/toolchain.json`.
- Generated operator guidance belongs in `docs/operations/` or runtime artifacts, not as additional markdown contracts inside `ops/`.
