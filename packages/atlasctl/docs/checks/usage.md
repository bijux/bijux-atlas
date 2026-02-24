# Atlasctl Check Command

## Pytest-Style UX

`atlasctl check run` is the canonical runner for policy checks.

Common usage:

- `atlasctl check run --domain repo`
- `atlasctl check run --category lint --domain docs`
- `atlasctl check run --id checks_repo_root_shape`
- `atlasctl check run --tag required --fail-fast`
- `atlasctl check run --include-internal --json`

Output modes:

- text (default)
- json (`--json`)
- jsonl (`--jsonl`)

All check and lint execution routes through one runner and one report envelope.

## Determinism Contract

- Check rows are sorted deterministically by canonical check id.
- JSON/jsonl envelopes are schema-versioned and stable.
- Runtime paths use explicit `run_id` and must not embed timestamps in path shape.

## Exit and Error Contract

- Success returns zero when no check fails or errors.
- Check failures and runner errors return non-zero.
- Runner errors are emitted in typed error fields in the report envelope.

## Lint Alias

`atlasctl lint <domain>` is a thin selector alias over checks:

- maps to `atlasctl check run --category lint --domain <domain>`
- uses the same selection logic
- uses the same report schema

## Triage

- `atlasctl check failures --last-run <path>`
- `atlasctl check triage-slow --last-run <path>`
- `atlasctl check triage-failures --last-run <path>`

`--last-run` accepts either:

- a report json file path
- a run directory path containing check-run json outputs
