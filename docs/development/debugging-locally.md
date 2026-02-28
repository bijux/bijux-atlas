# Debugging Locally

Owner: `platform`  
Type: `guide`  
Audience: `contributor`  
Reason to exist: provide local debugging entrypoints for fast issue isolation.

## Commands

```bash
make test
make stack-up
```

## Check Failure Triage

Use structured check reports to close failures with explicit owner and remediation mapping.

```bash
bijux dev atlas check run --group repo --json-report artifacts/evidence/checks/repo.json
bijux dev atlas checks failures --last-run artifacts/evidence/checks/repo.json --group repo --json
```

Relevant policy sources:

- `configs/policy/check-governance/metadata/owners.json`
- `configs/policy/check-intents.json`
- `configs/policy/check-remediation-map.json`
