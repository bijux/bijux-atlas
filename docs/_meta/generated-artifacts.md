# Generated artifacts

Generated documentation outputs are tooling artifacts, not reader-facing docs.

## What is generated

- `docs/_generated/docs-ledger.json`
- `docs/_generated/docs-contract-coverage.json`
- `docs/_generated/docs-quality-dashboard.json`
- `docs/_generated/search-index.json`
- `docs/_generated/governance-audit/*`

## How artifacts are generated

Run one of these commands from repository root:

```bash
make docs-generate
bijux dev atlas docs generate --strict
```

## Where artifacts live

- Source path: `docs/_generated/`
- Published path: not in reader nav, not indexed for search
- Access path: [Docs Dashboard](../governance/docs-dashboard.md)

## Usage rules

- Never link generated artifacts from user or operator guides.
- Use generated files only for contributor diagnostics and enforcement.
