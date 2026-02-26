# Ops Portability Matrix

- Owner: `bijux-atlas-operations`
- Purpose: `declare portability coverage expectations and proof artifacts for environment robustness`
- Consumers: `checks_ops_portability_environment_contract`
- Authority Tier: `machine`
- Audience: `contributors`

## Scope

This matrix defines the minimum portability and environment-robustness coverage that must remain true across ops assets.
It encodes coverage expectations using existing install profiles, stack profiles, load scenarios, observability goldens, and workflow isolation contracts.

## Platform Matrix

| Feature | Coverage source | Proof artifact |
| --- | --- | --- |
| macos-runner | workflow portability lane (external CI execution proof) | `artifacts/<run_id>/ci/` |
| minimal-linux-container | `ops/stack/profiles.json` (`minimal`) | `ops/stack/profiles.json` |
| local-only | `ops/k8s/install-matrix.json` (`local`) | `ops/k8s/install-matrix.json` |
| remote-execution | workflow routes via `bijux dev atlas` with run isolation | `.github/workflows/*.yml` |
| container-only-toolchain | pinned workflow/container actions and images | `ops/inventory/toolchain.json` |

## Environment Modes

| Feature | Coverage source | Proof artifact |
| --- | --- | --- |
| air-gapped-simulation | offline values + offline observability goldens | `ops/k8s/values/offline.yaml`, `ops/observe/contracts/goldens/offline/` |
| degraded-stack-mode | store outage scenarios and drills | `ops/load/scenarios/store-outage.json`, `ops/load/scenarios/store-outage-under-spike.json` |
| partial-dataset-mode | large dataset / selective dataset simulations | `ops/load/scenarios/large-dataset-simulation.json` |
| multi-registry | install matrix profile and values | `ops/k8s/install-matrix.json`, `ops/k8s/values/multi-registry.yaml` |
| alternate-storage-backend | optional redis scenario | `ops/load/scenarios/redis-optional.json` |

## Resource Pressure and Fault Simulation

| Feature | Coverage source | Proof artifact |
| --- | --- | --- |
| cpu-limited | noisy neighbor cpu throttle suite | `ops/load/scenarios/noisy-neighbor-cpu-throttle.json` |
| memory-limited | soak memory growth thresholds and drills | `ops/load/scenarios/soak-30m.json`, `ops/inventory/scenario-slo-map.json` |
| slow-network-simulation | rollout and store-outage degradation scenarios | `ops/load/scenarios/load-under-rollout.json`, `ops/load/scenarios/store-outage-under-spike.json` |
| time-skew-simulation | documented portability gap pending dedicated scenario | `ops/PORTABILITY_MATRIX.md` |
| missing-dependency-simulation | optional backend scenario (`redis-optional`) | `ops/load/scenarios/redis-optional.json` |

## Path Portability Invariants

- ops-authored contracts and inventories must use repo-relative forward-slash paths (`ops/...`, `docs/...`, `crates/...`, `makefiles/...`).
- Windows-style path separators (`\`) are forbidden in ops-authored path references.
- User-local absolute paths (for example `/Users/...`) are forbidden in ops-authored portability contracts and inventories.
- Portable-path validation is enforced by `checks_ops_portability_environment_contract`.

## Enforcement Links

- `checks_ops_portability_environment_contract`
- `checks_ops_workflow_routes_dev_atlas`
- `checks_ops_workflows_github_actions_pinned`
- `checks_ops_image_references_digest_pinned`

## Runtime Evidence Mapping

- Workflow portability and runner proofs: `artifacts/<run_id>/...` (external CI execution evidence; not committed)
- Offline observability proof goldens: `ops/observe/contracts/goldens/offline/*`
- Install and profile coverage: `ops/k8s/install-matrix.json`, `ops/stack/profiles.json`
- Load resilience/fault coverage: `ops/load/scenarios/*.json`, `ops/inventory/scenario-slo-map.json`
