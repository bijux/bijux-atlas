# Ops Contract

## Scope

- Governed surface: `ops/` and `ops/CONTRACT.md`.
- SSOT = bijux-dev-atlas contracts runner.
- Effects boundary: effect contracts require explicit runtime opt-in flags.
- Non-goals:
- This document does not replace executable contract checks.
- This document does not grant manual exception authority.

## Contract IDs

| ID | Title | Severity | Type(static/effect) | Enforced by | Artifacts |
| --- | --- | --- | --- | --- | --- |
| `OPS-DATASETS-001` | datasets manifest lock contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-DATASETS-002` | datasets fixture inventory contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-DATASETS-003` | datasets fixture drift promotion contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-DATASETS-004` | datasets release diff determinism contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-DATASETS-005` | datasets qc metadata summary contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-DATASETS-006` | datasets rollback policy contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-DATASETS-007` | datasets promotion rules contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-DATASETS-008` | datasets consumer interface contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-DATASETS-009` | datasets freeze policy contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-DATASETS-010` | datasets store layout contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-DATASETS-011` | datasets fixture archive lock contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-DATASETS-012` | datasets provenance fields contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-DATASETS-013` | datasets file type boundary contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-E2E-001` | e2e suites reference contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-E2E-002` | e2e smoke manifest contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-E2E-003` | e2e fixtures lock contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-E2E-004` | e2e realdata snapshot contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-E2E-005` | e2e taxonomy coverage contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-E2E-006` | e2e reproducibility enforcement contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-E2E-007` | e2e coverage matrix determinism contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-E2E-008` | e2e realdata scenario registry contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-E2E-009` | e2e surface artifact boundary contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-E2E-010` | e2e summary schema contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-E2E-E-001` | e2e effect smoke suite contract | `high` | `effect` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-E2E-E-002` | e2e effect realdata suite contract | `high` | `effect` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ENV-001` | environment overlay schema contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ENV-002` | environment profile completeness contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ENV-003` | environment key strictness contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ENV-004` | environment overlay merge determinism contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ENV-005` | environment prod safety toggles contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ENV-006` | environment ci effect restriction contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ENV-007` | environment base defaults contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ENV-008` | environment overlay key stability contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ENV-009` | environment overlays directory boundary contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-001` | inventory completeness contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-002` | inventory orphan files contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-003` | inventory duplicate source contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-004` | inventory authority tier contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-005` | inventory control graph contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-006` | inventory contract id format contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-007` | inventory gates registry contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-008` | inventory drills registry contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-009` | inventory owners registry contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-010` | inventory schema coverage contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-011` | inventory contracts listing pillar coverage contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-DEBT-001` | inventory contract debt registry contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-MAP-001` | inventory contract gate map coverage contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-MAP-002` | inventory contract gate map gate reference contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-MAP-003` | inventory contract gate map command surface contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-MAP-004` | inventory contract gate map effects annotation contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-MAP-005` | inventory contract gate map orphan gate contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-MAP-006` | inventory contract gate map orphan contract contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-MAP-007` | inventory contract gate map static purity contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-MAP-008` | inventory contract gate map effect kind contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-MAP-009` | inventory contract gate map explain coverage contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-MAP-010` | inventory contract gate map canonical order contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-MAP-011` | inventory contract gate map effect metadata contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-PILLARS-001` | inventory pillars registry contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-PILLARS-002` | inventory pillar directory contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-INV-PILLARS-003` | inventory pillar strictness contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-K8S-001` | k8s static chart render contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-K8S-002` | k8s values schema validation contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-K8S-003` | k8s install matrix completeness contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-K8S-004` | k8s forbidden object policy contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-K8S-005` | k8s rbac minimalism contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-K8S-006` | k8s pod security and probes contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-K8S-007` | k8s rollout safety contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-K8S-008` | k8s conformance suite contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-K8S-009` | k8s install matrix generated consistency contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-K8S-010` | k8s generated index determinism contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-K8S-E-001` | k8s effect chart defaults render contract | `high` | `effect` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-K8S-E-002` | k8s effect minimal values render contract | `high` | `effect` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-K8S-E-003` | k8s effect kubeconform contract | `high` | `effect` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-K8S-E-004` | k8s effect install matrix contract | `high` | `effect` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-K8S-E-005` | k8s effect rollout safety contract | `high` | `effect` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-K8S-E-006` | k8s effect tool versions contract | `high` | `effect` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-LOAD-001` | load scenario schema contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-LOAD-002` | load thresholds coverage contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-LOAD-003` | load pinned query lock contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-LOAD-004` | load baseline schema contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-LOAD-005` | load scenario to slo mapping contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-LOAD-006` | load drift report schema contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-LOAD-007` | load result schema sample contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-LOAD-008` | load cheap survival suite gate contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-LOAD-009` | load cold start p99 suite gate contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-LOAD-010` | load scenario slo impact mapping contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-LOAD-E-001` | load effect k6 execution contract | `high` | `effect` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-LOAD-E-002` | load effect thresholds report contract | `high` | `effect` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-LOAD-E-003` | load effect baseline comparison contract | `high` | `effect` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-META-001` | ops contracts map each contract id to a source file path | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-META-002` | ops contracts enforce io locality to ops surfaces | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-OBS-001` | observability alert rules contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-OBS-002` | observability dashboard golden contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-OBS-003` | observability telemetry golden profile contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-OBS-004` | observability readiness schema contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-OBS-005` | observability alert catalog generation contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-OBS-006` | observability slo burn-rate consistency contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-OBS-007` | observability public surface coverage contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-OBS-008` | observability telemetry index determinism contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-OBS-009` | observability drills manifest contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-OBS-010` | observability overload behavior contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-OBS-E-001` | observe effect metrics scrape contract | `high` | `effect` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-OBS-E-002` | observe effect trace structure contract | `high` | `effect` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-OBS-E-003` | observe effect alerts load contract | `high` | `effect` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-REPORT-001` | report schema ssot contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-REPORT-002` | report generated payload contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-REPORT-003` | report evidence levels contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-REPORT-004` | report diff structure contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-REPORT-005` | report readiness score determinism contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-REPORT-006` | report release evidence bundle contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-REPORT-007` | report historical comparison contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-REPORT-008` | report unified example contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-REPORT-009` | report canonical json output contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-REPORT-010` | report lane aggregation contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-001` | ops root allowed surface contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-002` | ops root markdown contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-003` | ops no shell scripts contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-004` | ops max directory depth contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-005` | ops filename policy contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-006` | ops generated gitignore policy contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-007` | ops generated example secret guard contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-008` | ops placeholder directory contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-009` | ops policy inventory coverage contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-010` | ops deleted doc name guard contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-011` | ops markdown allowlist contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-012` | ops pillar readme cardinality contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-013` | ops markdown allowlist inventory contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-014` | ops procedure text contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-015` | ops pillar markdown minimalism contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-016` | ops deleted markdown denylist contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-017` | ops directory contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-018` | ops generated lifecycle contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-019` | ops required domain files contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-020` | ops markdown budget contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-021` | ops docs ssot boundary contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-022` | ops contract document generation contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-023` | operations docs policy linkage contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-SURFACE-001` | ops root command surface required commands contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-SURFACE-002` | ops root command surface no hidden commands contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-SURFACE-003` | ops root command surface deterministic ordering contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-SURFACE-004` | ops root command surface effects declaration contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-SURFACE-005` | ops root command surface pillar grouping contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-SURFACE-006` | ops root command surface ad-hoc group guard contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-SURFACE-007` | ops root command surface purpose contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-SURFACE-008` | ops root command surface json output contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-SURFACE-009` | ops root command surface dry-run policy contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-ROOT-SURFACE-010` | ops root command surface artifacts policy contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-SCHEMA-001` | schema parseability contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-SCHEMA-002` | schema index completeness contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-SCHEMA-003` | schema naming contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-SCHEMA-004` | schema budget contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-SCHEMA-005` | schema evolution lock contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-SCHEMA-006` | schema id consistency contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-SCHEMA-007` | schema example validation contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-SCHEMA-008` | schema intent uniqueness contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-SCHEMA-009` | schema canonical formatting contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-SCHEMA-010` | schema example coverage contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-STACK-001` | stack toml profile contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-STACK-002` | stack service dependency contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-STACK-003` | stack version manifest contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-STACK-004` | stack dependency graph contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-STACK-005` | stack kind profile consistency contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-STACK-006` | stack ports inventory consistency contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-STACK-007` | stack health report generator contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-STACK-008` | stack command surface contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-STACK-009` | stack offline profile policy contract | `high` | `static` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-STACK-E-001` | stack effect kind cluster contract | `high` | `effect` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-STACK-E-002` | stack effect component rollout contract | `high` | `effect` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-STACK-E-003` | stack effect ports inventory contract | `high` | `effect` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-STACK-E-004` | stack effect health report contract | `high` | `effect` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |
| `OPS-STACK-E-005` | stack effect kind install smoke contract | `high` | `effect` | `bijux dev atlas contracts ops` | `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json` |

## Enforcement mapping

| Contract | Command(s) |
| --- | --- |
| `OPS-DATASETS-001` | `bijux dev atlas contracts ops --mode static` |
| `OPS-DATASETS-002` | `bijux dev atlas contracts ops --mode static` |
| `OPS-DATASETS-003` | `bijux dev atlas contracts ops --mode static` |
| `OPS-DATASETS-004` | `bijux dev atlas contracts ops --mode static` |
| `OPS-DATASETS-005` | `bijux dev atlas contracts ops --mode static` |
| `OPS-DATASETS-006` | `bijux dev atlas contracts ops --mode static` |
| `OPS-DATASETS-007` | `bijux dev atlas contracts ops --mode static` |
| `OPS-DATASETS-008` | `bijux dev atlas contracts ops --mode static` |
| `OPS-DATASETS-009` | `bijux dev atlas contracts ops --mode static` |
| `OPS-DATASETS-010` | `bijux dev atlas contracts ops --mode static` |
| `OPS-DATASETS-011` | `bijux dev atlas contracts ops --mode static` |
| `OPS-DATASETS-012` | `bijux dev atlas contracts ops --mode static` |
| `OPS-DATASETS-013` | `bijux dev atlas contracts ops --mode static` |
| `OPS-E2E-001` | `bijux dev atlas contracts ops --mode static` |
| `OPS-E2E-002` | `bijux dev atlas contracts ops --mode static` |
| `OPS-E2E-003` | `bijux dev atlas contracts ops --mode static` |
| `OPS-E2E-004` | `bijux dev atlas contracts ops --mode static` |
| `OPS-E2E-005` | `bijux dev atlas contracts ops --mode static` |
| `OPS-E2E-006` | `bijux dev atlas contracts ops --mode static` |
| `OPS-E2E-007` | `bijux dev atlas contracts ops --mode static` |
| `OPS-E2E-008` | `bijux dev atlas contracts ops --mode static` |
| `OPS-E2E-009` | `bijux dev atlas contracts ops --mode static` |
| `OPS-E2E-010` | `bijux dev atlas contracts ops --mode static` |
| `OPS-E2E-E-001` | `bijux dev atlas contracts ops --mode effect` |
| `OPS-E2E-E-002` | `bijux dev atlas contracts ops --mode effect` |
| `OPS-ENV-001` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ENV-002` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ENV-003` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ENV-004` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ENV-005` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ENV-006` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ENV-007` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ENV-008` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ENV-009` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-001` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-002` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-003` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-004` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-005` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-006` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-007` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-008` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-009` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-010` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-011` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-DEBT-001` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-MAP-001` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-MAP-002` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-MAP-003` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-MAP-004` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-MAP-005` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-MAP-006` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-MAP-007` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-MAP-008` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-MAP-009` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-MAP-010` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-MAP-011` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-PILLARS-001` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-PILLARS-002` | `bijux dev atlas contracts ops --mode static` |
| `OPS-INV-PILLARS-003` | `bijux dev atlas contracts ops --mode static` |
| `OPS-K8S-001` | `bijux dev atlas contracts ops --mode static` |
| `OPS-K8S-002` | `bijux dev atlas contracts ops --mode static` |
| `OPS-K8S-003` | `bijux dev atlas contracts ops --mode static` |
| `OPS-K8S-004` | `bijux dev atlas contracts ops --mode static` |
| `OPS-K8S-005` | `bijux dev atlas contracts ops --mode static` |
| `OPS-K8S-006` | `bijux dev atlas contracts ops --mode static` |
| `OPS-K8S-007` | `bijux dev atlas contracts ops --mode static` |
| `OPS-K8S-008` | `bijux dev atlas contracts ops --mode static` |
| `OPS-K8S-009` | `bijux dev atlas contracts ops --mode static` |
| `OPS-K8S-010` | `bijux dev atlas contracts ops --mode static` |
| `OPS-K8S-E-001` | `bijux dev atlas contracts ops --mode effect` |
| `OPS-K8S-E-002` | `bijux dev atlas contracts ops --mode effect` |
| `OPS-K8S-E-003` | `bijux dev atlas contracts ops --mode effect` |
| `OPS-K8S-E-004` | `bijux dev atlas contracts ops --mode effect` |
| `OPS-K8S-E-005` | `bijux dev atlas contracts ops --mode effect` |
| `OPS-K8S-E-006` | `bijux dev atlas contracts ops --mode effect` |
| `OPS-LOAD-001` | `bijux dev atlas contracts ops --mode static` |
| `OPS-LOAD-002` | `bijux dev atlas contracts ops --mode static` |
| `OPS-LOAD-003` | `bijux dev atlas contracts ops --mode static` |
| `OPS-LOAD-004` | `bijux dev atlas contracts ops --mode static` |
| `OPS-LOAD-005` | `bijux dev atlas contracts ops --mode static` |
| `OPS-LOAD-006` | `bijux dev atlas contracts ops --mode static` |
| `OPS-LOAD-007` | `bijux dev atlas contracts ops --mode static` |
| `OPS-LOAD-008` | `bijux dev atlas contracts ops --mode static` |
| `OPS-LOAD-009` | `bijux dev atlas contracts ops --mode static` |
| `OPS-LOAD-010` | `bijux dev atlas contracts ops --mode static` |
| `OPS-LOAD-E-001` | `bijux dev atlas contracts ops --mode effect` |
| `OPS-LOAD-E-002` | `bijux dev atlas contracts ops --mode effect` |
| `OPS-LOAD-E-003` | `bijux dev atlas contracts ops --mode effect` |
| `OPS-META-001` | `bijux dev atlas contracts ops --mode static` |
| `OPS-META-002` | `bijux dev atlas contracts ops --mode static` |
| `OPS-OBS-001` | `bijux dev atlas contracts ops --mode static` |
| `OPS-OBS-002` | `bijux dev atlas contracts ops --mode static` |
| `OPS-OBS-003` | `bijux dev atlas contracts ops --mode static` |
| `OPS-OBS-004` | `bijux dev atlas contracts ops --mode static` |
| `OPS-OBS-005` | `bijux dev atlas contracts ops --mode static` |
| `OPS-OBS-006` | `bijux dev atlas contracts ops --mode static` |
| `OPS-OBS-007` | `bijux dev atlas contracts ops --mode static` |
| `OPS-OBS-008` | `bijux dev atlas contracts ops --mode static` |
| `OPS-OBS-009` | `bijux dev atlas contracts ops --mode static` |
| `OPS-OBS-010` | `bijux dev atlas contracts ops --mode static` |
| `OPS-OBS-E-001` | `bijux dev atlas contracts ops --mode effect` |
| `OPS-OBS-E-002` | `bijux dev atlas contracts ops --mode effect` |
| `OPS-OBS-E-003` | `bijux dev atlas contracts ops --mode effect` |
| `OPS-REPORT-001` | `bijux dev atlas contracts ops --mode static` |
| `OPS-REPORT-002` | `bijux dev atlas contracts ops --mode static` |
| `OPS-REPORT-003` | `bijux dev atlas contracts ops --mode static` |
| `OPS-REPORT-004` | `bijux dev atlas contracts ops --mode static` |
| `OPS-REPORT-005` | `bijux dev atlas contracts ops --mode static` |
| `OPS-REPORT-006` | `bijux dev atlas contracts ops --mode static` |
| `OPS-REPORT-007` | `bijux dev atlas contracts ops --mode static` |
| `OPS-REPORT-008` | `bijux dev atlas contracts ops --mode static` |
| `OPS-REPORT-009` | `bijux dev atlas contracts ops --mode static` |
| `OPS-REPORT-010` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-001` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-002` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-003` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-004` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-005` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-006` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-007` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-008` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-009` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-010` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-011` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-012` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-013` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-014` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-015` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-016` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-017` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-018` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-019` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-020` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-021` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-022` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-023` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-SURFACE-001` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-SURFACE-002` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-SURFACE-003` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-SURFACE-004` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-SURFACE-005` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-SURFACE-006` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-SURFACE-007` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-SURFACE-008` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-SURFACE-009` | `bijux dev atlas contracts ops --mode static` |
| `OPS-ROOT-SURFACE-010` | `bijux dev atlas contracts ops --mode static` |
| `OPS-SCHEMA-001` | `bijux dev atlas contracts ops --mode static` |
| `OPS-SCHEMA-002` | `bijux dev atlas contracts ops --mode static` |
| `OPS-SCHEMA-003` | `bijux dev atlas contracts ops --mode static` |
| `OPS-SCHEMA-004` | `bijux dev atlas contracts ops --mode static` |
| `OPS-SCHEMA-005` | `bijux dev atlas contracts ops --mode static` |
| `OPS-SCHEMA-006` | `bijux dev atlas contracts ops --mode static` |
| `OPS-SCHEMA-007` | `bijux dev atlas contracts ops --mode static` |
| `OPS-SCHEMA-008` | `bijux dev atlas contracts ops --mode static` |
| `OPS-SCHEMA-009` | `bijux dev atlas contracts ops --mode static` |
| `OPS-SCHEMA-010` | `bijux dev atlas contracts ops --mode static` |
| `OPS-STACK-001` | `bijux dev atlas contracts ops --mode static` |
| `OPS-STACK-002` | `bijux dev atlas contracts ops --mode static` |
| `OPS-STACK-003` | `bijux dev atlas contracts ops --mode static` |
| `OPS-STACK-004` | `bijux dev atlas contracts ops --mode static` |
| `OPS-STACK-005` | `bijux dev atlas contracts ops --mode static` |
| `OPS-STACK-006` | `bijux dev atlas contracts ops --mode static` |
| `OPS-STACK-007` | `bijux dev atlas contracts ops --mode static` |
| `OPS-STACK-008` | `bijux dev atlas contracts ops --mode static` |
| `OPS-STACK-009` | `bijux dev atlas contracts ops --mode static` |
| `OPS-STACK-E-001` | `bijux dev atlas contracts ops --mode effect` |
| `OPS-STACK-E-002` | `bijux dev atlas contracts ops --mode effect` |
| `OPS-STACK-E-003` | `bijux dev atlas contracts ops --mode effect` |
| `OPS-STACK-E-004` | `bijux dev atlas contracts ops --mode effect` |
| `OPS-STACK-E-005` | `bijux dev atlas contracts ops --mode effect` |

## Output artifacts

- `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.json`
- `artifacts/run/<run_id>/gates/contracts/ops/<profile>/<mode>/ops.inventory.json`

## Contract to Gate mapping

- Gate: `contracts::ops`
- Aggregate gate: `contracts::all`

## Exceptions policy

- No exceptions are allowed by this document.
