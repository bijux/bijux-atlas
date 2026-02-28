# Bijux Plugin Conformance Checklist

- Owner: `docs-governance`

Use this checklist for each `bijux-<subsystem>` plugin release.

- [ ] Binary name follows `bijux-<subsystem>`.
- [ ] Plugin is discoverable via `$PATH` scan as `bijux-*` executable.
- [ ] `--bijux-plugin-metadata` returns valid JSON and exits `0`.
- [ ] Metadata includes required fields: `name`, `version`, `compatible_umbrella`, `build_hash`.
- [ ] Shared exit code policy is implemented.
- [ ] Global flags implemented: `--json`, `--quiet`, `--verbose`, `--trace`.
- [ ] Shared env vars honored: `BIJUX_LOG_LEVEL`, `BIJUX_CACHE_DIR`.
- [ ] Shared config path resolution follows `docs/plugin-spec.md`.
- [ ] Completion contract implemented via `completion <shell>`.
- [ ] Help output uses standard section order and template.
- [ ] Subcommand namespace avoids reserved umbrella verbs.
- [ ] Machine error contract is stable and emitted on stderr in `--json` mode.
- [ ] Unknown fields are rejected for machine contracts where required.
- [ ] Contract tests cover metadata, error schema, and exit codes.

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
