# Commands

## Surface

Stable command families:
- `ops`
- `docs`
- `configs`
- `policies`
- `check`

Top-level command namespace is intentionally limited to these families.

## Global Flags

- `--json`: force machine JSON output mode.
- `--quiet`: suppress rendered stdout payloads.
- `--fail-fast`: enforce fail-fast behavior for run-like checks and strict doctor/validate flows.
- `--repo-root <PATH>`: set repository root once and propagate to command handlers.

## Examples

- `bijux dev atlas check list --suite ci_fast`
- `bijux dev atlas check run --suite ci_fast --fail-fast`
- `bijux dev atlas docs doctor --json`
- `bijux dev atlas configs validate --json --fail-fast`
- `bijux dev atlas ops validate --json`
- `bijux dev atlas policies report --json`

## Exit Codes

See exit-codes.md`.
