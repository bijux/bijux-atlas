# Observability Acceptance Checklist

- Owner: `bijux-atlas-operations`

## Required Checks

- [ ] `make ops-observability-pack-lint`
- [ ] `make observability-pack-test`
- [ ] `make observability-pack-drills`
- [ ] `make telemetry-verify`
- [ ] `artifacts/observability/pack-conformance-report.json` generated
- [ ] `artifacts/observability/drill-conformance-report.json` generated

## Release Notes

- [ ] Telemetry contract version changes documented.
- [ ] Breaking telemetry changes include migration notes.
