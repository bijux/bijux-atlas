# SLO Baseline Update Policy

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

## Purpose

Define when SLO targets and baseline expectations may be adjusted.

## Allowed reasons

- Metric definition changes that materially alter interpretation.
- Product-scope changes that add/remove endpoint classes.
- Sustained measured behavior that invalidates previous baseline assumptions.

## Required process

- Update `configs/ops/slo/slo.v1.json` and `docs/operations/slo/SLOS.md` together.
- Add a dated entry in `docs/operations/slo/CHANGELOG.md` with rationale and blast radius.
- Include explicit before/after values and the measurement window used.
- Run `make ci-slo-config-validate ci-slo-metrics-contract ci-slo-docs-drift`.

## Not allowed

- Relaxing targets to bypass transient failures without approval.
- Silent target changes without changelog and policy references.
