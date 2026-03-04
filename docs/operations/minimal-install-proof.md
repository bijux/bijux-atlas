---
title: Minimal install proof
owner: platform
stability: stable
last_reviewed: 2026-03-05
---

# Minimal install proof

A minimal install proof requires:
- successful `ops install --plan --evidence`
- successful `ops render --target kind --evidence`
- successful `ops validate --evidence`
- generated artifacts under `artifacts/ops/evidence/<run_id>/`

Attach these with `release/evidence/manifest.json` for review.
