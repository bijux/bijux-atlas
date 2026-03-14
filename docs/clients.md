---
title: Clients
audience: user
type: reference
stability: stable
owner: product
last_reviewed: 2026-03-14
related:
  - docs/api/index.md
  - docs/tutorials/clients/index.md
---

# Clients

Client SDK products live under `packages/`.

## Canonical client locations

- `packages/bijux-atlas-python/`

## Python client layout

- `python/`: package source (`atlas_client`)
- `examples/`: runnable usage examples
- `tests/`: product tests
- `docs/`: client documentation
- `notebooks/`: notebook assets

## Policy

- Root `clients/` is forbidden.
- Repository automation for clients must run through `bijux-dev-atlas clients ...`.
- Python and notebook files are only allowed in approved package and crate zones defined in `configs/governance/allowed-nonrust.json`.
