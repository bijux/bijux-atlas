# Ops

- Owner: `bijux-atlas-operations`
- Authority Tier: `explanatory`
- Audience: `mixed`

## What

Canonical operational filesystem surface and SSOT entrypoint for ops workflows.
`bijux dev atlas ops ...` is the runtime control plane for validation, rendering,
install/status flows, and deterministic ops generation.

Reference contract: `ops/CONTRACT.md`.
Runbook index: `ops/INDEX.md`.

## Directory map

- `ops/stack/`: local stack dependency bring-up.
- `ops/k8s/`: chart, install profiles, and k8s-only gates.
- `ops/observe/`: observability pack, contracts, and drills.
- `ops/load/`: k6 suites, scenarios, contracts, baselines.
- `ops/datasets/`: dataset manifest, pinning, QC, promotion.
- `ops/e2e/`: composition-only scenarios over stack/observe/load/datasets.
- `bijux dev atlas ops ...` and `make` wrappers: operator entrypoints (no direct script surface).
- canonical local workflow runs through `bijux-dev-atlas` (`ops render`, `ops install`, `ops status`).
- `ops/inventory/meta/`: ownership/surface/contracts metadata.
- `ops/schema/`: ops JSON schemas.
- `ops/_generated.example/`: deterministic generated ops outputs committed to git.
- `artifacts/`: runtime evidence outputs (gitignored).

## Run

- `cargo run -p bijux-dev-atlas -- ops doctor --format json`
- `cargo run -p bijux-dev-atlas -- ops validate --format json`
- `cargo run -p bijux-dev-atlas -- ops list-tools --allow-subprocess --format json`
- `cargo run -p bijux-dev-atlas -- ops verify-tools --allow-subprocess --format json`
- `cargo run -p bijux-dev-atlas -- ops render --target kind --check --format json`
- `cargo run -p bijux-dev-atlas -- ops install --kind --plan --allow-subprocess --allow-write`
- `cargo run -p bijux-dev-atlas -- ops status --target pods --allow-subprocess --format json`
- `cargo run -p bijux-dev-atlas -- ops generate pins-index --allow-write --format json`
- `make ops-kind-up`
- `make ops-kind-down`
- `make ops-status`

## SSOT Files

These files are treated as frozen ops inventory inputs and are validated by
`bijux dev atlas ops doctor` / `ops validate`:

- `ops/stack/generated/version-manifest.json`
- `ops/inventory/pins.yaml`
- `ops/inventory/toolchain.json`
- `ops/stack/profiles.json`
