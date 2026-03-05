# Real Data E2E Execution Report

This report explains how to reproduce and inspect real-data end-to-end runs executed by `bijux-dev-atlas`.

## Inputs

- Workflow contract: `configs/tutorials/real-data-runs-workflow.json`
- Run catalog: `configs/tutorials/real-data-runs.json`
- Dataset fetch specs: `tutorials/datasets/*/fetch-spec.json`

## Generated Evidence

- Command log: `artifacts/tutorials/real-data-examples/command-log.json`
- Run verification summary: `artifacts/tutorials/real-data-examples/check-results-heavy-partial.json`
- Per-run artifacts: `artifacts/tutorials/runs/<run_id>/`

## Canonical Execution Commands

```bash
cargo run -p bijux-dev-atlas -- tutorials real-data list --format json
cargo run -p bijux-dev-atlas -- tutorials real-data doctor --format json
cargo run -p bijux-dev-atlas -- tutorials real-data run-all --profile local --format json
```

## What To Check In Results

1. Dataset bytes are non-trivial for large scenarios.
2. `outputs_ok` is `true` for each completed run in the summary report.
3. Each run directory contains all six expected files.
4. `manifest.json` and `bundle.sha256` exist and match the run evidence bundle.

## Interpretation Guidance

- A run can be marked complete in command output but still fail final validation if checksum or required files are missing.
- Heavy dataset runs are network-sensitive; rerun per-run commands for only failed IDs.
- Use `--no-fetch` for deterministic replay on already-downloaded bytes.

## Related Pages

- [Real Data E2E Tutorial](../../tutorials/real-data-e2e.md)
- [Real Data Runs Report](real-data-runs.md)
