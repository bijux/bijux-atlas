# Atlasctl Boundaries

## Effect Boundaries

- Commands must not execute subprocesses directly; use `atlasctl.core.exec` (process/effect wrappers in `core`).
- Commands must not write files directly; use `atlasctl.core.fs` and allowed roots policy.
- Allowed write roots are repository-governed output locations only (for example `artifacts/`, `docs/_generated/`, `ops/_generated_committed/`, and other roots explicitly allowed by policy checks).
- External tools must be invoked through explicit wrappers and deterministic arguments; no hidden side effects.

## Invariants

- `atlasctl` is the source of truth entrypoint for repository tooling flows.
- Makefiles are delegation surfaces only and must not reimplement business/tooling logic.
- Top-level package directories must each be explainable in one sentence.
- Module size policy is max 400 LOC per file, with explicit exceptions tracked in review notes or policy docs.

## Package And Naming Policy

- Canonical top-level architecture map: `atlasctl/core`, `atlasctl/cli`, `atlasctl/commands`, `atlasctl/checks`, `atlasctl/contracts`.
- Command modules follow `command.py` per domain package.
- Duplicate domains are forbidden: `checks/` is canonical; `check/` remains legacy-only compatibility until removal.
- `layout_checks/` are classified as legacy checks; net-new checks go in first-class `atlasctl/checks`.

## Error, Exit, And Logging Policy

- `ScriptError` is the only user-facing exception type.
- Process exit codes are defined in a single enum module: `atlasctl/exit_codes.py`.
- Ad-hoc raw `sys.exit(1)` usage in command paths is disallowed.
- Logging emits structured events via shared logger/emitter layers; no ad-hoc `print()` outside CLI/output boundaries.

## JSON Output Contract

- JSON command payloads must share a base envelope: `{schema_version, tool, status, ...}`.
- Schemas and validators are owned by `atlasctl/contracts`.
- Machine-consumed output must be deterministic.

## Golden Tests Policy

Golden-test the following surfaces at minimum:

- `--help` output for public commands.
- Commands inventory output (`commands` JSON/text surface).
- Generated surface docs and command maps.
- Boundary/policy report outputs used by CI checks.

## Deprecation And Migration Policy

- Deprecated flags/options must include an owner and explicit expiry date.
- Default deprecation window: two minor releases unless an exception is documented.
- Expired deprecations must fail CI until removed.
- `packages/atlasctl/MIGRATION_MAP.md` must be treated as a temporary monolith and split into smaller domain-focused docs.
- New migration entries should be added to split docs; monolith updates are transitional only.
