# bijux-atlas-cli

Plugin-facing CLI for atlas operations.

## Canonical Binary

- `bijux-atlas`

## Command Model

- `bijux-atlas atlas <subcommand>`: canonical atlas namespace command path.
- `bijux-atlas --bijux-plugin-metadata`: plugin handshake.
- `bijux-atlas serve`: container compatibility entrypoint that starts `atlas-server`.

## Standard Flags

- `--json`
- `--quiet`
- `--verbose`
- `--trace`
