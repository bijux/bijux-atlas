# Scenarios As Evidence

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`

## Run

```bash
bijux dev atlas ops scenario run --scenario minimal-single-node --plan --format json
bijux dev atlas ops scenario run --scenario artifact-integrity --evidence --allow-write --format json
```

## Interpret

- `rows[].run_id` is deterministic for `scenario + mode`.
- `rows[].required_evidence_files` lists mandatory artifacts.
- Evidence mode writes result JSON and markdown summary.
