# Telemetry Compatibility

- Owner: `bijux-atlas-operations`

## Contract

Telemetry contract changes are additive by default.
Breaking changes require:
- contract version bump
- migration notes in docs
- regenerated server telemetry artifacts

## Required process

1. Update `ops/observability/contract/metrics-contract.json` or `docs/contracts/TRACE_SPANS.json`.
2. Run `make telemetry-contracts`.
3. Run `make telemetry-verify`.
4. Document migration in `docs/operations/observability/acceptance-checklist.md` release notes section.

## Verification

- `make telemetry-contracts`
- `make telemetry-verify`
