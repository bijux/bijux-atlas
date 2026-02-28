# Kind Profile

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/stack/profiles.json`, `ops/stack/kind/cluster-dev.yaml`, `ops/stack/kind/cluster-perf.yaml`, `ops/stack/kind/cluster-small.yaml`

## What

Defines the canonical local kind profiles used by stack and k8s operations.

## Why

Kind profile names and file paths must stay stable so docs, smoke runs, and release checks all refer to the same cluster shapes.

## Profiles

- `dev`: `ops/stack/kind/cluster-dev.yaml`
- `perf`: `ops/stack/kind/cluster-perf.yaml`
- `small`: `ops/stack/kind/cluster-small.yaml`

## How To Verify

```bash
cargo test -q -p bijux-dev-atlas --test ops_k8s_contracts -- --nocapture
```

Expected output: kind profile docs and stack profile manifests stay aligned.
