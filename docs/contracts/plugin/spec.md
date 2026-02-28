# Bijux Plugin Specification v1

Concept IDs: concept.plugin-contract

- Owner: `bijux-atlas-cli`

This document defines the v1 contract between the Bijux umbrella runner and subsystem plugins.
Umbrella/runtime SSOT: `docs/ecosystem/umbrella-plugin-contract-v1.md`.

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

All Bijux plugins must support these global flags:

- `--json`: machine-readable output mode.
- `--quiet`: suppress non-essential output.
- `--verbose`: increase human-readable detail.
- `--trace`: maximum diagnostic detail (intended for debugging).

Shared environment variables:

- `BIJUX_LOG_LEVEL`: logging verbosity override (`error|warn|info|debug|trace`).
- `BIJUX_CACHE_DIR`: shared cache directory for plugin-managed caches.

### `--json` Convention

When `--json` is enabled:

- success output must be structured JSON.
- error output must be structured JSON on stderr where possible.
- field names must remain stable across patch/minor versions.

## Shared Config Path Resolution

All plugins MUST resolve config locations in this order:

1. Workspace config: `./.bijux/config.toml`.
2. User config:
   - `$XDG_CONFIG_HOME/bijux/config.toml` when `XDG_CONFIG_HOME` is set.
   - Otherwise `$HOME/.config/bijux/config.toml` when `HOME` is set.
   - Fallback `./.bijux/config.toml`.
3. Cache directory:
   - `BIJUX_CACHE_DIR` when set and non-empty.
   - `$XDG_CACHE_HOME/bijux` when `XDG_CACHE_HOME` is set.
   - Otherwise `$HOME/.cache/bijux`.
   - Fallback `./.bijux/cache`.

## Completion Contract

- Plugins must expose shell completion generation via a `completion` subcommand.
- Canonical form:

```bash
bijux-<subsystem> completion <shell>
```

- Supported shells must include at least `bash`, `zsh`, and `fish`.
- Completion output is always script content on stdout and MUST be side-effect free.

## Help Formatting Standard

- Help output MUST use a consistent clap help template across plugins.
- Sections order MUST be:
  1. Name/version
  2. About
  3. Usage
  4. Options
  5. Commands
  6. After-help notes (including environment variable references)

## Subcommand Namespace Rule

- Plugin subcommands MUST be namespaced under the plugin command and MUST NOT rely on umbrella-owned top-level verbs.
- Reserved umbrella verbs include: `plugin`, `plugins`, `doctor`, `config`.
- Example: `bijux-atlas dataset validate` is valid; exposing umbrella verbs in plugin command space is invalid.

## Error Output Schema (Machine Contract)

When `--json` is enabled and a command fails, stderr MUST emit stable JSON with this schema:

```json
{
  "code": "machine_stable_error_code",
  "message": "human readable summary",
  "details": {
    "key": "stable string value"
  }
}
```

Rules:

- `code` MUST be stable across patch/minor versions.
- Unknown fields MUST NOT be emitted.
- `details` values MUST be scalar strings for compatibility.
- Exit code MUST still follow the shared process exit code contract.

## Plugin Conformance Checklist

Conformance checklist lives at `docs/plugin-conformance-checklist.md` and MUST pass before a plugin is considered Bijux-compatible.

## Compatibility Notes

- Existing alias binaries (legacy names) MAY exist, but canonical plugin identity is the `bijux-*` name.
- Umbrella integrations must prefer canonical names during discovery and execution.

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

- [Contracts Overview](../INDEX.md)
- [SSOT Workflow](../ssot-workflow.md)
- [Terms Glossary](../../_style/terms-glossary.md)
