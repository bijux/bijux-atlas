# CLI Troubleshooting

## Config validation failed

- Run `bijux-dev-atlas configs print --format json`.
- Compare keys to `configs/cli/config.schema.json`.

## Completion not loading

- Rebuild command inventory with `bijux-dev-atlas help --format text`.
- Verify shell startup references your completion configuration.

## Unexpected command failure

- Inspect `artifacts/cli/command-audit.json` and telemetry output paths.
- Run command directly with `--output-format json` for machine-readable diagnostics.
