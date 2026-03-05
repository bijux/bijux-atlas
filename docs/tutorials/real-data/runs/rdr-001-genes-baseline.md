---
title: Genes Baseline Tiny
audience: user
type: tutorial
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
---

# Genes Baseline Tiny

## Dataset intro

- Run ID: rdr-001-genes-baseline
- Dataset ID: genes-baseline
- Size tier: tiny
- Ingest mode: full-rebuild

## Ingest steps

1. 'bijux-dev-atlas tutorials real-data fetch --run-id rdr-001-genes-baseline --format json'
2. 'bijux-dev-atlas tutorials real-data ingest --run-id rdr-001-genes-baseline --profile local --format json'

## Query pack

- gene_count_total

Run command: 'bijux-dev-atlas tutorials real-data query-pack --run-id rdr-001-genes-baseline --format json'

## Evidence links

- Run artifacts directory: artifacts/tutorials/runs/rdr-001-genes-baseline/
- Expected artifacts:
- ingest-report.json
- dataset-summary.json
- Export command: 'bijux-dev-atlas tutorials real-data export-evidence --run-id rdr-001-genes-baseline --profile local --format json'

## How to reproduce locally

1. 'bijux-dev-atlas tutorials real-data plan --run-id rdr-001-genes-baseline --format json'
2. 'bijux-dev-atlas tutorials real-data fetch --run-id rdr-001-genes-baseline --format json'
3. 'bijux-dev-atlas tutorials real-data ingest --run-id rdr-001-genes-baseline --profile local --format json'
4. 'bijux-dev-atlas tutorials real-data query-pack --run-id rdr-001-genes-baseline --format json'
5. 'bijux-dev-atlas tutorials real-data export-evidence --run-id rdr-001-genes-baseline --profile local --format json'

## Known limitations

- Dataset source currently uses a fixed fetch specification in tutorials/datasets/genes-baseline/fetch-spec.json.
- Results depend on the selected runtime profile and local machine capacity.

## Performance notes

- Record timing from artifacts/tutorials/runs/rdr-001-genes-baseline/query-results-summary.json after each run.
- Compare latency for warm and cold runs when available.

## Operational profile

- Expected resource profile class: small
- Recommended local profile: local

## Data governance note

- Source URL: https://example.org/datasets/genes-baseline.csv
- Retrieval method: https-get
- License note: Public domain demonstration dataset

## Troubleshooting

- If fetch fails, run 'bijux-dev-atlas tutorials real-data doctor --format json' and verify tool health.
- If ingest fails on profile constraints, retry with '--profile minimal' and inspect ingest report.
- If query pack fails, inspect artifacts/tutorials/runs/rdr-001-genes-baseline/query-results-summary.json for query-level errors.

## Change impact

Changes to dataset fetch specs, ingest mapping, query-pack definitions, or evidence schema can invalidate prior results for this run.
