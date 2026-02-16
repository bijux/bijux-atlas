# Bijux-Atlas Plugin Mode

`bijux-atlas` is a plugin executable intended to be launched by the `bijux` umbrella.

## Invocation Model

- Canonical plugin binary: `bijux-atlas`.
- Umbrella dispatch path: `bijux atlas <atlas-command...>` -> `bijux-atlas <atlas-command...>`.
- Atlas namespace path: `bijux-atlas atlas <atlas-command...>`.

## Metadata Handshake

`bijux-atlas --bijux-plugin-metadata` returns JSON with:

- `name`
- `version`
- `compatible_umbrella`
- `build_hash`

## Logging and Output

Global plugin flags are supported:

- `--json`
- `--quiet`
- `--verbose`
- `--trace`

For server mode, canonical startup is `bijux-atlas atlas serve`, which forwards logging intent to `atlas-server` via `BIJUX_LOG_LEVEL` / `RUST_LOG`.

## Container and Chart Alignment

- Docker entrypoint: `/app/bijux-atlas atlas serve`
- Helm default command (`values.yaml`): `server.command = ["/app/bijux-atlas","atlas","serve"]`
