---
title: Clients Doctrine
audience: contributor
type: policy
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - clients
  - automation
  - governance
---

# Clients doctrine

Client SDK code is product code. Repository automation for clients is owned by `bijux-dev-atlas`.

## Rules

- Python is permitted for the Python SDK product itself under approved client paths.
- Repository documentation generation and verification for clients must run through `bijux-dev-atlas clients ...`.
- Client docs and examples must not depend on local ad hoc scripts.
- Client verification outputs must be deterministic and machine-readable.

## Directory purpose

- `clients/atlas-client/atlas_client`: Python SDK implementation.
- `clients/atlas-client/tests`: SDK tests.
- `clients/atlas-client/examples`: SDK usage examples.
- `clients/atlas-client/docs`: generated and maintained client docs.
- `clients/`: product SDK assets only, not repo-level automation tooling.

## Required replacement command surface

- `bijux-dev-atlas clients list`
- `bijux-dev-atlas clients verify`
- `bijux-dev-atlas clients docs-generate --client atlas-client`
- `bijux-dev-atlas clients docs-verify --client atlas-client`
- `bijux-dev-atlas clients examples-verify --client atlas-client`
- `bijux-dev-atlas clients schema-verify --client atlas-client`
- `bijux-dev-atlas clients doctor --client atlas-client`

