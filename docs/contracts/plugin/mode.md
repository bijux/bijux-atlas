# Bijux-Atlas Plugin Mode

Concept IDs: concept.plugin-contract

- Owner: `bijux-atlas-cli`

`bijux-atlas` is a plugin executable intended to be launched by the `bijux` umbrella.

Canonical page: [`docs/contracts/plugin/spec.md`](spec.md)

## Invocation Model

- Canonical plugin binary: `bijux-atlas`.
- Umbrella dispatch path: `bijux atlas <atlas-command...>` -> `bijux-atlas <atlas-command...>`.
- Atlas namespace path: `bijux-atlas atlas <atlas-command...>`.
- Dev control-plane dispatch path: `bijux dev atlas <dev-command...>` -> `bijux-dev-atlas <dev-command...>`.

## Canonical Command Surface Lists

- Runtime plugin command list: `crates/bijux-atlas-cli/docs/CLI_COMMAND_LIST.md`
- Dev control-plane command list: `crates/bijux-dev-atlas/docs/CLI_COMMAND_LIST.md`
- Dev ops command list snapshot: `crates/bijux-dev-atlas/docs/OPS_COMMAND_LIST.md`
- Dev configs command list snapshot: `crates/bijux-dev-atlas/docs/CONFIGS_COMMAND_LIST.md`

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

## What

Defines a stable contract surface for this topic.

## Why

Prevents ambiguity and drift across CLI, API, and operations.

## Scope

Applies to atlas contract consumers and producers.

## Non-goals

Does not define internal implementation details beyond the contract surface.

## Contracts

Use the rules in this page as the normative contract.

## Failure modes

Invalid contract input is rejected with stable machine-readable errors.

## Examples

```bash
$ make ssot-check
```

Expected output: a zero exit code and "contract artifacts generated" for successful checks.

## How to verify

Run `make docs docs-freeze ssot-check` and confirm all commands exit with status 0.

## See also

- [Contracts Overview](../index.md)
- [SSOT Workflow](../ssot-workflow.md)
- [Terms Glossary](../../_style/terms-glossary.md)
