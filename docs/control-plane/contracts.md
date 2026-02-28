# Contracts Control Plane

- Owner: `atlas-maintainers`
- Stability: `stable`

## Exit Codes

- `0`: all selected contracts passed.
- `1`: one or more non-required contracts failed.
- `2`: usage error, including invalid wildcard filters or missing required flags.
- `3`: internal runner error.
- `4`: one or more required contracts failed.

## CI Mode

- Use `bijux dev atlas contracts ... --ci` for CI-facing runs.
- `--ci` forces the CI profile and disables ANSI color in human output.
- CI runs still emit machine artifacts through `--json-out`, `--junit-out`, and `artifacts/contracts/**`.

## Artifacts

- Per-domain JSON reports: `artifacts/contracts/<domain>/<profile>/<mode>/<run_id>/<domain>.json`
- Unified multi-domain JSON report: `artifacts/contracts/<run scope>/unified.json`
- Unified multi-domain markdown report: `artifacts/contracts/<run scope>/unified.md`
- Panic evidence: `artifacts/contracts/<domain>/<profile>/<mode>/<run_id>/panics.json`

## Registry Integrity

- Contract IDs must be unique across domains.
- Every contract must map to at least one test.
- Every test ID must map to exactly one contract ID.
- Required contracts must declare at least one lane.

## See Also

- [Root Contract](../../CONTRACT.md)
- [Docs Contract](../CONTRACT.md)
- [Lane Guarantees](../operations/release/lane-guarantees.md)
