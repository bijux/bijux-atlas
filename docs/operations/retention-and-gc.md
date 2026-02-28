# Retention And GC

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `ops`
- Stability: `evolving`

## Why

Artifact stores grow quickly during local and CI ops runs. GC keeps the store bounded without deleting catalog-reachable or pinned datasets.

## What

GC is exposed only via CLI commands:

- `atlas gc plan`: dry run, prints deletion candidates.
- `atlas gc apply --confirm`: deletes only unreachable and unpinned artifacts, then writes a GC report.

Pins are SSOT at `ops/inventory/pins.yaml`:

- `dataset_ids`: canonical dataset keys to preserve.
- `artifact_hashes`: exact artifact hashes to preserve.

Contract schema is `docs/reference/contracts/GC_POLICY.json`.

## How

Use make entrypoints for ops smoke:

```bash
make ops-gc-smoke
```

Manual planning/apply:

```bash
cargo run -p bijux-atlas-cli -- atlas gc plan \
  --store-root artifacts/e2e-datasets \
  --catalog artifacts/e2e-datasets/catalog.json \
  --pins ops/inventory/pins.yaml

cargo run -p bijux-atlas-cli -- atlas gc apply \
  --store-root artifacts/e2e-datasets \
  --catalog artifacts/e2e-datasets/catalog.json \
  --pins ops/inventory/pins.yaml \
  --confirm
```

## Failure modes

- Running in server container/runtime is rejected.
- `apply` without `--confirm` is rejected.
- Deletions outside store root are rejected.
- Partial failures are recorded in GC report artifacts under `gc_reports/`.

## Air-gapped retention

For air-gapped installs, keep currently promoted releases pinned and run only `atlas gc plan` in approval workflows before cleanup windows.
