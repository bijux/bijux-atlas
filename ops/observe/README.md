# ops/observe

- Owner: `bijux-atlas-observability`
- Purpose: `observability contracts, drills, and generated verification indexes`
- Consumers: `bijux dev atlas ops observe commands, checks_ops_domain_contract_structure`

Observability assets (dashboards, alerts, rules, otel, prometheus) in the target layout.

## Contracts

- `ops/observe/slo-definitions.json`
- `ops/observe/alert-catalog.json`
- `ops/observe/telemetry-drills.json`
- `ops/observe/readiness.json`
- `ops/observe/generated/telemetry-index.json`

## Drill Templates

- `ops/observe/drills/templates/incident-template.md`

Placeholder extension directories tracked with `.gitkeep`: `ops/observe/alerts`, `ops/observe/dashboards`, `ops/observe/prom`, `ops/observe/otel`, `ops/observe/rules`.
