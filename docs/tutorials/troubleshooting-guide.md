---
title: Troubleshooting Guide
audience: user
type: guide
stability: stable
owner: docs-governance
last_reviewed: 2026-03-04
tags:
  - troubleshooting
related:
  - docs/api/troubleshooting.md
  - docs/operations/incident-response.md
---

# Troubleshooting Guide

## Approach

1. Identify failing surface (ingest, query, release, docs).
2. Collect command output and error code.
3. Map to contract/check owner.
4. Apply fix in owning layer and re-run validations.

## References

- [API troubleshooting](../api/troubleshooting.md)
- [Incident response](../operations/incident-response.md)


## Tutorial dataset checks

- `tutorials/scripts/validate_example_dataset.py`
- `tutorials/scripts/reproducibility_check.sh`
- `tutorials/scripts/integrity_check.sh`
