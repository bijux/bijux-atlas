# Platform Handover

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`
- Stability: `stable`

## Scope

Operational handover checklist for running atlas, upgrading releases, adding datasets, and debugging latency.

## Operate

```bash
$ make ops-local-full-stack
$ make ops-report
$ make ops-readiness-scorecard
```

## Upgrade Release

```bash
$ make ops-release-update DATASET=medium
$ make ops-drill-upgrade-under-load
$ make ops-drill-rollback-under-load
```

## Add Dataset

```bash
$ make ops-datasets-fetch
$ make ops-publish DATASET=medium
$ make ops-dataset-qc
```

## Debug Latency

```bash
$ make ops-load-smoke
$ make ops-perf-report
$ make ops-incident-repro-kit
```

Expected output: artifacts under `artifacts/ops/<run-id>/` and incident bundle under `artifacts/incident/<run-id>/`.
