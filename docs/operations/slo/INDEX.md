# SLO Contract Index

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `mixed`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`
- Stability: `stable`

## What

Single landing page for Bijux Atlas SLI/SLO/SLA policy and machine-checked targets.

## Why

Provide one canonical navigation entry for SLO policy, contracts, generated docs, and verification targets.

## Scope

- SLI class definitions and endpoint mapping
- SLO target definitions and policies
- SLO docs drift and config validation gates

## Non-goals

- Defining external SLA commitments for v1
- Replacing area-specific observability contracts

## Contracts

- `configs/ops/slo/classes.json`
- `configs/ops/slo/sli.schema.json`
- `configs/ops/slo/slo.schema.json`
- `configs/ops/slo/slo.v1.json`

## Failure modes

- SLO policy references undefined metrics or labels.
- Class-to-endpoint mapping drifts from API surface.
- Contract changes ship without documented policy change.

## How to verify

- `make ci-slo-config-validate`
- `make ci-slo-metrics-contract`
- `make ci-slo-docs-drift`

## See also

- [Glossary](GLOSSARY.md)
- [Scope](SCOPE.md)
- [Non-goals](NON_GOALS.md)
- [Change policy](CHANGE_POLICY.md)
- [SLIs](SLIS.md)
- [SLO Targets](SLOS.md)
- [SLO Release Gate](RELEASE_GATE.md)
- [Baseline Update Policy](BASELINE_UPDATE_POLICY.md)
- [SLA Policy (v1)](SLA_POLICY.md)
- [SLA Exclusions (Planned)](SLA_EXCLUSIONS_PLANNED.md)
- [SLA Decision ADR Template](SLA_DECISION_ADR_TEMPLATE.md)
- [Why These SLIs](WHY_THESE_SLIS.md)
- [What We Do Not Measure Yet](WHAT_WE_DONT_MEASURE_YET.md)
