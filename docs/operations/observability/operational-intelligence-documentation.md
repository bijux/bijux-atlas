# Operational Intelligence Documentation

- Owner: `bijux-atlas-operations`
- Type: `guide`
- Audience: `operator`
- Stability: `stable`

## Scope

This guide describes how Atlas uses dashboards, metrics, traces, and logs to drive production decisions.

## Inputs

- Dashboard registry: `ops/observe/dashboard-registry.json`
- Dashboard validation contract: `ops/observe/contracts/dashboard-json-validation-contract.json`
- Dashboard metadata schema: `ops/observe/dashboard-metadata.schema.json`

## Commands

- `bijux-dev-atlas observe dashboards list --format json`
- `bijux-dev-atlas observe dashboards verify --format json`
- `bijux-dev-atlas observe dashboards explain --format json`
