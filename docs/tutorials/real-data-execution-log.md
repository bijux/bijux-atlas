# Real Data Execution Log

This page records the exact reproducible command sequence for real-data examples.

Source of truth:
- `configs/tutorials/real-data-runs-workflow.json`

Execution log artifact:
- `artifacts/tutorials/real-data-examples/command-log.json`

All execution commands are from `bijux-dev-atlas`:

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
