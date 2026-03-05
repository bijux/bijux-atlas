# Command Surface

Canonical command documentation for `bijux-dev-atlas`.

## Stable Families

- `ops`
- `docs`
- `configs`
- `contract`
- `policies`
- `check`

## Global Flags

- `--json`: force machine JSON output mode.
- `--quiet`: suppress rendered stdout payloads.
- `--fail-fast`: enforce fail-fast behavior for run-like checks and strict doctor/validate flows.
- `--repo-root <PATH>`: set repository root once and propagate to command handlers.

## Contract Pages

- command surface policy: `command-surface.md`
- control-plane contract: `control-plane-contracts.md`
- error and exit behavior: `errors-and-exit-codes.md`

## Examples

- `bijux dev atlas check list --suite ci_fast`
- `bijux dev atlas check run --suite ci_fast --fail-fast`
- `bijux dev atlas docs doctor --json`
- `bijux dev atlas contract all --format json`
- `bijux dev atlas configs validate --json --fail-fast`
- `bijux dev atlas ops validate --json`
- `bijux dev atlas policies report --json`
