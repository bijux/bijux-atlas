# Telemetry Compatibility

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## Contract

Telemetry contract changes are additive by default.
Breaking changes require:
- contract version bump
- compatibility notes in docs
- regenerated server telemetry artifacts

## Required process

1. Update `docs/contracts/metrics.md` or `docs/contracts/tracing.md`.
2. Run `make telemetry-contracts`.
3. Run `make telemetry-verify`.
4. Document compatibility notes in `docs/operations/observability/acceptance-checklist.md` release notes section.

## Verification

- `make telemetry-contracts`
- `make telemetry-verify`
- `make ops-observability-validate`
