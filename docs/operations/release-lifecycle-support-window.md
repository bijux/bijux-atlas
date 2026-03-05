---
title: Release lifecycle and support window
owner: bijux-atlas-operations
stability: stable
last_reviewed: 2026-03-05
---

# Release lifecycle and support window

- Active release line: `v0.1`
- Support window policy: follow image/chart lifecycle declarations in release specs.
- Deprecated channels must preserve manifest + checksums + provenance for audit continuity.

Minimum retained files per release:
- `artifacts/release/<version>/manifest.json`
- `artifacts/release/<version>/bundle.tar`
- `release/signing/checksums.json`
- `release/provenance.json`
