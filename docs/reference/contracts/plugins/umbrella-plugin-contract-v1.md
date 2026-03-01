# Umbrella Plugin Contract v1

- Owner: `docs-governance`

Single source of truth for how `bijux` umbrella interacts with plugins.

This document refines and references `docs/plugin-spec.md`.
If a conflict exists, this file is authoritative for umbrella/plugin runtime behavior.

## Discovery

- Plugin binaries must be named `bijux-<subsystem>`.
- Umbrella discovers plugins from `$PATH`.
- Atlas plugin binary is `bijux-atlas`.

## Dispatch

- Umbrella invocation: `bijux atlas <args...>`
- Effective exec: `bijux-atlas <args...>`

## Metadata Handshake

`bijux-atlas --bijux-plugin-metadata` must return JSON with fields:

- `schema_version`
- `name`
- `version`
- `compatible_umbrella`
- `compatible_umbrella_min`
- `compatible_umbrella_max_exclusive`
- `build_hash`

## Compatibility Enforcement

- Umbrella must validate plugin compatibility before dispatch when possible.
- Plugin also accepts `--umbrella-version <v>` and rejects incompatible versions.

## Shared Flags

Plugins support shared top-level flags:

- `--json`
- `--quiet`
- `--verbose`
- `--trace`

## Command Namespace

- Atlas operational commands are namespaced under `atlas`.
- Top-level plugin commands are reserved for handshake/completion/version ergonomics.

## Determinism

- Completion output must be deterministic.
- Command-surface drift is gated by CI using `docs/CLI_COMMAND_LIST.md`.

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
$ make contracts
```

Expected output: a zero exit code and "contract artifacts generated" for successful checks.

## How to verify

Run `make docs docs-freeze ssot-check` and confirm all commands exit with status 0.

## See also

- [Contracts Overview](../index.md)
- [SSOT Workflow](../../../_internal/governance/contract-ssot-workflow.md)
- [Terms Glossary](../../../glossary.md)
