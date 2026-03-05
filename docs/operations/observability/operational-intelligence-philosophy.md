# Operational Intelligence Philosophy

- Owner: `bijux-atlas-operations`
- Type: `guide`
- Audience: `operator`
- Stability: `stable`

## Purpose

Define how Atlas turns telemetry into clear operational decisions.

## Principles

- Prefer decision-driving signals over volume-heavy telemetry.
- Keep runtime, ingest, query, and ops views separated and composable.
- Treat dashboard drift as an operational regression.
- Keep dashboards tied to stable metric contracts.
- Require reproducible incident analysis from dashboard artifacts.

## Decision Model

1. Detect: identify budget or latency deviation.
2. Classify: map impact to runtime, ingest, query, or platform.
3. Verify: correlate metrics with traces and logs.
4. Mitigate: apply runbook-based controls.
5. Confirm: validate recovery in SLO and error dashboards.
