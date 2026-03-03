# Ops Simulation Contracts

- Owner: `docs-governance`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: define the executable install simulation checks for the kind-backed operations surface.

## Contract IDs

- `OPS-SIM-001`: `profile-baseline` installs in kind, reaches readiness, and produces `ops-install.json`.
- `OPS-SIM-002`: `ci` installs in kind, reaches readiness, and produces `ops-install.json`.
- `OPS-SIM-003`: `offline` installs in kind, reaches readiness, and produces `ops-install.json`.
- `OPS-SIM-004`: `perf` installs in kind, reaches readiness, and produces `ops-install.json` even when autoscaling is disabled.
- `OPS-SIM-005`: uninstall produces `ops-cleanup.json` and leaves no leftover namespaced resources.

## Evidence

- `ops-install.json`
- `ops-smoke.json`
- `ops-cleanup.json`
- `ops-simulation-summary.json`
- `ops-debug-bundle-*.json`

## Reproduce locally

```bash
bijux dev atlas ops kind up --allow-subprocess --allow-write --format json
bijux dev atlas ops helm install --profile profile-baseline --cluster kind --allow-subprocess --allow-write --allow-network --format json
bijux dev atlas ops smoke --profile profile-baseline --cluster kind --allow-subprocess --allow-write --allow-network --format json
bijux dev atlas ops helm uninstall --profile profile-baseline --cluster kind --allow-subprocess --allow-write --allow-network --format json
```

## Failure model

- A failed readiness wait is an install contract failure.
- A non-`200` health endpoint is a smoke contract failure.
- Any leftover object in the namespace is a cleanup contract failure.
