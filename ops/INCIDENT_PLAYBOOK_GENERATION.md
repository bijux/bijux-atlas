# Incident Playbook Generation

- Authority Tier: `tier2`
- Audience: `operators`
- Owner: `bijux-atlas-operations`
- Purpose: `define how incident playbooks are produced from drill inventory, runbook policies, and control graph mappings`
- Consumers: `checks_ops_human_workflow_maturity`

## Inputs
- `ops/inventory/drills.json`
- `ops/inventory/drill-contract-links.json`
- `ops/inventory/control-graph.json`
- `ops/RUNBOOK_GENERATION_FROM_GRAPH.md`

## Generation Rules
- Every generated incident playbook must map to a drill id and at least one control graph node.
- Playbook outputs must live under `docs/operations/`.
- Curated example generation evidence must be committed under `ops/_generated.example/`.

## Outputs
- `docs/operations/EXAMPLE_INCIDENT_WALKTHROUGH.md`
- `ops/_generated.example/incident-playbook-generation-report.json`

## Enforcement Links
- `checks_ops_human_workflow_maturity`
