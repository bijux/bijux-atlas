# SLO Contract Index

- Owner: `bijux-atlas-operations`
- Stability: `stable`

## Purpose

Single landing page for Bijux Atlas SLI/SLO/SLA policy and machine-checked targets.

## Entry points

- `make ci-slo-config-validate`
- `make ci-slo-metrics-contract`

## Documents

- [Glossary](GLOSSARY.md)
- [Scope](SCOPE.md)
- [Non-goals](NON_GOALS.md)
- [Change policy](CHANGE_POLICY.md)

## Source of truth

- `configs/ops/slo/classes.json`
- `configs/ops/slo/sli.schema.json`
- `configs/ops/slo/slo.schema.json`
- `configs/ops/slo/slo.v1.json`

## Failure modes

- SLO policy references undefined metrics or labels.
- Class-to-endpoint mapping drifts from API surface.
- Contract changes ship without documented policy change.
