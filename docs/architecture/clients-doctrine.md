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

- `crates/bijux-atlas-client-python/python/atlas_client`: Python SDK implementation.
- `crates/bijux-atlas-client-python/tests`: SDK tests.
- `crates/bijux-atlas-client-python/examples`: SDK usage examples.
- `crates/bijux-atlas-client-python/docs`: generated and maintained client docs.
- Root `clients/` is forbidden.

## Required replacement command surface

- `bijux-dev-atlas clients list`
- `bijux-dev-atlas clients verify`
- `bijux-dev-atlas clients docs-generate --client atlas-client`
- `bijux-dev-atlas clients docs-verify --client atlas-client`
- `bijux-dev-atlas clients examples-verify --client atlas-client`
- `bijux-dev-atlas clients examples-run --client atlas-client`
- `bijux-dev-atlas clients schema-verify --client atlas-client`
- `bijux-dev-atlas clients compat-matrix verify --client atlas-client`
- `bijux-dev-atlas clients doctor --client atlas-client`

Client docs generation source-of-truth:

- `configs/clients/atlas-client-docs.json`
- `configs/openapi/v1/openapi.snapshot.json`

Usage examples location decision:

- Canonical usage examples live under `crates/bijux-atlas-client-python/examples/usage/`.
