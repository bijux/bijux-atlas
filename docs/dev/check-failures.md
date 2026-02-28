# Check Failures Closure Guide

Use this workflow to turn failing checks into tracked work:

1. Run checks with JSON report output:

```bash
bijux dev atlas check run --group repo --json-report artifacts/evidence/checks/repo.json
```

2. Summarize failures:

```bash
bijux dev atlas checks failures --last-run artifacts/evidence/checks/repo.json
bijux dev atlas checks failures --last-run artifacts/evidence/checks/repo.json --group repo --json
```

3. Resolve in SSOT metadata:
- `configs/policy/check-governance/metadata/owners.json`
- `configs/policy/check-intents.json`
- `configs/policy/check-remediation-map.json`

Failure classes:
- `policy`: command/layout/process constraints not met.
- `contract`: output/docs/schema drift.
- `hygiene`: structural or ownership quality issues.

Remediation rule:
- Every failing check must map to an owner and a concrete fix path before merge.
