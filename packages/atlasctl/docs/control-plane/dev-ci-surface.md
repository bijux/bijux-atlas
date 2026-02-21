# DEV/CI Surface

This document is the SSOT for the supported DEV/CI control-plane entrypoints.

## Stable Front Door

- CI one-liner: `atlasctl dev ci run`
- Developer shortcuts:
- `atlasctl dev fmt`
- `atlasctl dev lint`
- `atlasctl dev check`
- `atlasctl dev test`
- `atlasctl dev test --all`
- `atlasctl dev test --contracts`
- `atlasctl dev coverage`
- `atlasctl dev audit`

## Stability Policy

- `atlasctl dev ci run` is the stable public CI entrypoint.
- `atlasctl suite run ci` is internal plumbing and not the stable public CI front door.
- Makefiles must call stable `atlasctl` entrypoints only.
- Makefiles must not call the internal suite engine directly.

## Gate Semantics

- Fast default gates do not run the full `repo` checks group.
- Full gates (`*-all`) include `atlasctl check run repo`.
- Canonical semantics:
- `fmt` = fmt-only
- `fmt-all` = fmt + `atlasctl check run repo`
- `lint` = lint-only
- `lint-all` = lint + `atlasctl check run repo`
- `test` = test-only
- `test-all` = test + `atlasctl check run repo`
- `check` = cargo check only
- `check-all` = cargo check + `atlasctl check run repo`
- `audit` = cargo deny only
- `audit-all` = cargo deny + `atlasctl check run repo`
- `docs` = docs lane only
- `docs-all` = docs lane + `atlasctl check run repo`
- `ops` = ops lane only
- `ops-all` = ops lane + `atlasctl check run repo`
- Optional `--and-checks` exists for explicit fast-lane opt-in, but `*-all` is the canonical full variant.

## Human vs CI Use

- Humans should run the stable front door commands above.
- CI workflows should use the same stable front door (`atlasctl dev ci run`) for CI suite execution.

## CI Run Contract

- `atlasctl dev ci run` emits:
- JSON report
- JUnit XML
- text summary
- Artifacts default to `artifacts/evidence/ci/<run_id>/...`.
- Supports lane filtering via `--lane` (for example `rust`, `fmt`, `lint`, `test`, `contracts`, `docs`, `ops`).
- Supports execution mode flags: `--fail-fast` or `--keep-going`.
- Isolation is required by default; `--no-isolate` is debug-only.
