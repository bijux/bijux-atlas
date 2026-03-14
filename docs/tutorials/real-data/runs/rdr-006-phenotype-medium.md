---
title: Phenotype Medium
audience: user
type: tutorial
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
---

# Phenotype Medium

## Dataset intro

- Run ID: rdr-006-phenotype-medium
- Dataset ID: phenotype-medium
- Size tier: medium
- Ingest mode: incremental-merge

## Ingest steps

1. 'bijux-dev-atlas tutorials real-data fetch --run-id rdr-006-phenotype-medium --format json'
2. 'bijux-dev-atlas tutorials real-data ingest --run-id rdr-006-phenotype-medium --profile local --format json'

## Query pack

- phenotype_match_count

Run command: 'bijux-dev-atlas tutorials real-data query-pack --run-id rdr-006-phenotype-medium --format json'

## Evidence links

- Run artifacts directory: artifacts/tutorials/runs/rdr-006-phenotype-medium/
- Expected artifacts:
- ingest-report.json
- dataset-summary.json
- query-results-summary.json
- Export command: 'bijux-dev-atlas tutorials real-data export-evidence --run-id rdr-006-phenotype-medium --profile local --format json'

## How to reproduce locally

1. 'bijux-dev-atlas tutorials real-data plan --run-id rdr-006-phenotype-medium --format json'
2. 'bijux-dev-atlas tutorials real-data fetch --run-id rdr-006-phenotype-medium --format json'
3. 'bijux-dev-atlas tutorials real-data ingest --run-id rdr-006-phenotype-medium --profile local --format json'
4. 'bijux-dev-atlas tutorials real-data query-pack --run-id rdr-006-phenotype-medium --format json'
5. 'bijux-dev-atlas tutorials real-data export-evidence --run-id rdr-006-phenotype-medium --profile local --format json'

## Known limitations

- Dataset source currently uses a fixed fetch specification in ops/tutorials/datasets/phenotype-medium/fetch-spec.json.
- Results depend on the selected runtime profile and local machine capacity.

## Performance notes

- Record timing from artifacts/tutorials/runs/rdr-006-phenotype-medium/query-results-summary.json after each run.
- Compare latency for warm and cold runs when available.

## Operational profile

- Expected resource profile class: medium
- Recommended local profile: local

## Data governance note

- Source URL: https://example.org/datasets/phenotype-medium.csv
- Retrieval method: https-get
- License note: CC0 data sample

## Troubleshooting

- If fetch fails, run 'bijux-dev-atlas tutorials real-data doctor --format json' and verify tool health.
- If ingest fails on profile constraints, retry with '--profile minimal' and inspect ingest report.
- If query pack fails, inspect artifacts/tutorials/runs/rdr-006-phenotype-medium/query-results-summary.json for query-level errors.

## Change impact

Changes to dataset fetch specs, ingest mapping, query-pack definitions, or evidence schema can invalidate prior results for this run.
