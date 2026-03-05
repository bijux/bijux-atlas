---
title: Tutorial: Deploy Runtime Locally
audience: user
type: guide
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - tutorial
  - runtime
  - local
related:
  - docs/operations/run-locally.md
  - docs/operations/operator-quickstart.md
---

# Tutorial: Deploy Runtime Locally

## Goal

Run Atlas runtime on a local machine for development and tutorial flows.

## Steps

1. Build runtime binaries.
2. Start runtime with `configs/examples/runtime/server-minimal.toml`.
3. Confirm `/health` endpoint returns success.

## Expected result

Runtime accepts API requests from local SDK and CLI clients.
