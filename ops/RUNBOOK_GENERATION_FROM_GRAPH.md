# Runbook Generation From Graph

- Authority Tier: `machine`
- Audience: `contributors`
- Owner: `bijux-atlas-operations`
- Purpose: `define how operator runbooks are derived from inventory control graph nodes and edges`
- Consumers: `checks_ops_human_workflow_maturity`

## Inputs
- `ops/inventory/control-graph.json`
- `ops/inventory/drill-contract-links.json`
- `ops/inventory/authority-index.json`

## Generation Rules
- Runbooks must identify source graph nodes and consumer edges.
- Generated runbook sections must map to domain nodes and escalation classes.
- Manual edits to generated runbooks are forbidden unless marked as explanatory overlays.

## Outputs
- Generated runbook index under `docs/operations/`
- Graph-linked runbook references in incident workflows

## Enforcement Links
- `checks_ops_human_workflow_maturity`
