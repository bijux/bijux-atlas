---
title: Contributor Troubleshooting Guide
audience: contributor
type: troubleshooting
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - troubleshooting
---

# Contributor troubleshooting guide

## Governance check fails

Run:

```bash
bijux-dev-atlas governance check --format json
```

Inspect failed rule IDs and fix referenced files.

## Governance validate fails

Run:

```bash
bijux-dev-atlas governance validate --format json
```

Resolve errors in `checks_inventory`, `governance_docs_validation`, and `contributor_guidelines_validation` sections.
