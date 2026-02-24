# bijux-dev-atlas

Control-plane binary for repository governance under `bijux dev atlas ...`.

## Control Plane Philosophy

- No scripts as control-plane SSOT.
- Command behavior flows through crate APIs, not shell orchestration.
- Outputs are deterministic and contract-driven.
- Execution is hermetic by default: network/subprocess/write/git are opt-in.

## Stable Families

- `ops`
- `docs`
- `configs`
- `policies`
- `check`

## Common Flags

- `--json`
- `--quiet`
- `--fail-fast`
- `--repo-root`

## Contracts

- Command surface: `docs/CLI_COMMAND_LIST.md`
- Examples and behavior: `docs/COMMANDS.md`
- Exit codes: `docs/EXIT_CODES.md`
- Control-plane contract: `docs/CONTRACT.md`
