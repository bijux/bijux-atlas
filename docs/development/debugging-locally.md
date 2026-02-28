# Debugging Locally

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@d489531b`
- Reason to exist: provide the canonical local triage playbook.

## Reproduce

Start by reproducing the failure with explicit lane commands.

```bash
make test
bijux dev atlas check run --group repo --json-report artifacts/evidence/checks/repo.json
```

## Inspect Artifacts

Inspect structured outputs first; do not debug from intuition.

```bash
bijux dev atlas checks failures --last-run artifacts/evidence/checks/repo.json --group repo --json
```

Policy sources commonly used during triage:

- `configs/policy/check-governance/metadata/owners.json`
- `configs/policy/check-intents.json`
- `configs/policy/check-remediation-map.json`

## Fix

- Apply the smallest change that restores the violated contract.
- Re-run the exact reproducer command.
- Re-run broader suite only after focused check is green.

## Verify Success

- Focused reproducer command passes.
- Expected evidence artifact updates are deterministic.
- `make test` remains green before merge.

## What to Read Next

- [Control-plane](../control-plane/index.md)
- [CI Overview](ci-overview.md)
- [Contributing](contributing.md)

## Document Taxonomy

- Audience: `contributor`
- Type: `guide`
- Stability: `stable`
- Owner: `platform`
