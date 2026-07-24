# Ops

`ops/` is the repository-owned source of truth for Atlas operational inputs, inventories, schemas, fixtures, and generated examples.

- Intent: keep `ops/` as operational data, schemas, inventories, fixtures, and generated evidence examples.
- Machine validation entrypoint: `bijux-atlas-dev ops validate --format json`.
- Focused execution entrypoints: `bijux-atlas-dev ops profiles ...`, `bijux-atlas-dev ops render ...`, `bijux-atlas-dev ops install ...`, and `bijux-atlas-dev ops stack ...`.
- Operator walkthroughs and procedures live in `docs/bijux-atlas-ops/`.
- Maintainer workflows and governance live in `docs/bijux-atlas-dev/`.
- Product runtime and interface contracts live in `docs/bijux-atlas/`.

## Root Docs

- `ops/README.md`: what `ops/` is for and what does not belong here.
- `ops/CONTRACT.md`: durable rules, authorities, and evidence expectations.
- `ops/INDEX.md`: canonical directory map for the live ops surface.
- `ops/ERRORS.md`: root-level error vocabulary for ops validation and repo-law failures.
- `ops/SSOT.md`: markdown policy for `ops/`.

## Design Rules

- Path should tell you whether a file is authored truth, schema, fixture, generated example, or release evidence.
- `ops/inventory/` and `ops/schema/` are the root authorities; the other top-level directories are domain-owned inputs or evidence fixtures.
- Inventories under `ops/inventory/` describe operational authorities; they do not replace validation output.
- Generated examples under `ops/_generated.example/` are illustrative evidence mirrors, not authored truth.
- Runtime effect commands require explicit opt-in flags; static inventory and schema checks do not.
- Markdown is intentionally tiny. Narrative and policy prose is limited to the five root docs; deeper `ops/` paths should stay machine-readable.

## Authority Layers

| Layer | Primary paths | Review question |
| --- | --- | --- |
| authored authority | `ops/inventory/`, `ops/schema/`, domain inputs | what rules, identities, scenarios, and required evidence were declared? |
| executable interpretation | `crates/bijux-atlas-ops/` | how are those assets loaded, related, validated, and projected into typed results? |
| effect orchestration | `crates/bijux-atlas-dev/` | which command may read, write, spawn, contact, or mutate an external target? |
| observed evidence | `artifacts/` and governed release packet inputs | what actually ran, against which target, with which result and artifact binding? |
| public explanation | `docs/bijux-atlas-ops/` | how should an operator use and interpret the contract without overstating proof? |

Do not edit generated evidence to repair authored policy, add narrative Markdown
under a machine-owned domain, or treat an inventory record as a successful run.
Change the owning input, execute the named validation or scenario, and retain
the resulting report under the run or release identity that produced it.

## Release And Runbook Scope

- Minimum release evidence lives in data, not prose: `ops/inventory/contracts-map.json`, `ops/inventory/authority-index.json`, `ops/load/suites/suites.json`, `ops/observe/drills.json`, and `ops/report/generated/readiness-score.json`.
- Runbook generation is driven by data authorities: `ops/inventory/control-graph.json`, `ops/k8s/install-matrix.json`, `ops/stack/profile-intent.json`, and `ops/inventory/toolchain.json`.
- Generated operator guidance belongs in `docs/bijux-atlas-ops/`,
  `docs/bijux-atlas-dev/`, or runtime artifacts, not as additional Markdown
  contracts inside `ops/`.
