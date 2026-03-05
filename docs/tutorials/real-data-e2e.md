# Real Data E2E Tutorial

This tutorial records the end-to-end execution path on real public datasets using `bijux-dev-atlas` commands only.

## Scope

- Real source systems: ENA and NCBI
- Execution path: download/fetch -> ingest -> query-pack -> export-evidence
- Artifacts written under `artifacts/tutorials/`

## Commands Used

```bash
cargo run -p bijux-dev-atlas -- tutorials real-data list --format json
cargo run -p bijux-dev-atlas -- tutorials real-data doctor --format json
cargo run -p bijux-dev-atlas -- tutorials real-data run-all --profile local --format json
```

Per-run replay:

```bash
cargo run -p bijux-dev-atlas -- tutorials real-data fetch --run-id <run_id> --profile local --format json
cargo run -p bijux-dev-atlas -- tutorials real-data ingest --run-id <run_id> --profile local --no-fetch --format json
cargo run -p bijux-dev-atlas -- tutorials real-data query-pack --run-id <run_id> --profile local --no-fetch --format json
cargo run -p bijux-dev-atlas -- tutorials real-data export-evidence --run-id <run_id> --profile local --no-fetch --format json
```

## Reports And Logs

- Command sequence definition: `configs/tutorials/real-data-runs-workflow.json`
- Command log: `artifacts/tutorials/real-data-examples/command-log.json`
- Heavy-run verification report: `artifacts/tutorials/real-data-examples/check-results-heavy-partial.json`
- Run outputs root: `artifacts/tutorials/runs/<run_id>/`
- Downloaded datasets root: `artifacts/tutorials/datasets/<dataset>/`

## Latest Results

From `artifacts/tutorials/real-data-examples/check-results-heavy-partial.json`:

| Run ID | Dataset Bytes | Outputs OK |
| --- | ---:| --- |
| `rdr-001-genes-baseline` | 222202406 | `true` |
| `rdr-002-transcripts-baseline` | 227260208 | `true` |
| `rdr-003-variants-mini` | 342790903 | `true` |
| `rdr-004-expression-medium` | 161093628 | `true` |
| `rdr-005-population-medium` | 575985811 | `true` |
| `rdr-006-phenotype-medium` | 121574068 | `true` |
| `rdr-007-assembly-large-sample` | 353452929 | `true` |
| `rdr-008-annotation-large-sample` | 1049720328 | `true` |
| `rdr-010-combined-release` | 9999 | `true` |

## Result Interpretation

- `outputs_ok=true` means all required run artifacts exist:
  `ingest-report.json`, `dataset-summary.json`, `query-results-summary.json`, `evidence-bundle.json`, `manifest.json`, `bundle.sha256`.
- The byte counts confirm this execution exercised real large files, including a ~1GB-class dataset.
- Missing runs in a partial report indicate interrupted or failed download/verification and should be rerun with `fetch` then `ingest/query-pack/export-evidence`.
