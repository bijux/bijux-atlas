# Bijux Plugin Specification v1

This document defines the v1 contract between the Bijux umbrella runner and subsystem plugins.

## Binary Naming Convention

- Plugin binary names MUST follow: `bijux-<subsystem>`.
- Example: `bijux-atlas`.

## Plugin Discovery Rule

- The umbrella discovers plugins by scanning `$PATH` for executables matching `bijux-*`.
- Discovery does not execute commands beyond metadata handshake unless explicitly invoked.

## Version Handshake Protocol

- A plugin MUST implement:

```bash
bijux-<subsystem> --bijux-plugin-metadata
```

- The command MUST return JSON to stdout and exit `0`.
- The command MUST avoid side effects and must not require external state.

## Required Metadata Fields

Handshake JSON MUST contain:

- `name`: plugin canonical binary name.
- `version`: plugin semantic version.
- `compatible_umbrella`: umbrella version range supported by plugin.
- `build_hash`: immutable build identifier.

Recommended optional fields:

- `description`
- `homepage`

## Standard Exit Codes (Shared)

All Bijux tools share these process exit codes:

- `0`: success.
- `2`: usage / invalid arguments.
- `3`: validation / policy failure.
- `4`: runtime dependency failure (I/O, DB, network, environment).
- `10`: internal error / unexpected failure.

## Standard CLI Flags

All Bijux plugins SHOULD support these global flags:

- `--json`: machine-readable output mode.
- `--quiet`: suppress non-essential output.
- `--verbose`: increase human-readable detail.
- `--trace`: maximum diagnostic detail (intended for debugging).

### `--json` Convention

When `--json` is enabled:

- success output should be structured JSON.
- error output should be structured JSON on stderr where possible.
- field names should remain stable across patch/minor versions.

## Compatibility Notes

- Existing alias binaries (legacy names) MAY exist, but canonical plugin identity is the `bijux-*` name.
- Umbrella integrations should prefer canonical names during discovery and execution.
