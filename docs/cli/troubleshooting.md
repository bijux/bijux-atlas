# CLI Troubleshooting

## Config validation failed

- Run `python3 tools/cli/atlas_cli_runner.py --print-effective-config`.
- Compare keys to `configs/cli/config.schema.json`.

## Completion not loading

- Re-run `tools/cli/shell-integration/install-completions.sh`.
- Verify shell startup references.

## Unexpected command failure

- Inspect `artifacts/cli/command-audit.json` and telemetry output paths.
- Run command directly with `--output-format json` for machine-readable diagnostics.
