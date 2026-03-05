---
title: Governance Enforcement Troubleshooting
audience: operator
type: troubleshooting
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - troubleshooting
---

# Governance enforcement troubleshooting

## Common violations

- `required file is missing`
  - Restore the missing file or correct the rule path.
- `checks registry has no checks entries`
  - Ensure `configs/governance/checks.registry.json` has a non-empty `checks` array.
- `scenario registry has no scenarios entries`
  - Ensure `ops/e2e/scenarios/scenarios.json` has a non-empty `scenarios` array.
- `docs navigation is missing or empty`
  - Ensure `mkdocs.yml` contains a non-empty `nav` section.

## Diagnostic commands

```bash
bijux-dev-atlas governance rules --format json
bijux-dev-atlas governance check --format json
bijux-dev-atlas governance explain GOV-RULE-010 --format json
```
