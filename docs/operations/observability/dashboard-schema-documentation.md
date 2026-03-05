# Dashboard Schema Documentation

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`

## Schema Source

- `ops/schema/observe/dashboard.schema.json`

## Required Fields

- `title`
- `uid`
- `schemaVersion`
- `version`
- `panels`
- `tags`

## Panel Rules

- Every panel must include `title` and `type`.
- Metric panels must include query `targets[].expr` values.
- Dashboard JSON must pass schema validation before merge.
