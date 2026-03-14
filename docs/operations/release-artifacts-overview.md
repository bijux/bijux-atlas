---
title: Release artifacts overview
owner: bijux-atlas-operations
stability: stable
last_reviewed: 2026-03-05
---

# Release artifacts overview

- Crates: publishable crates in `ops/release/crates-v0.1.toml`
- Images: publishable images in `ops/release/images-v0.1.toml`
- Ops chart and bundle: `ops/release/ops-v0.1.toml`
- Release manifest: `artifacts/release/<version>/manifest.json`
- Release bundle: `artifacts/release/<version>/bundle.tar`
- Provenance and checksums: `ops/release/provenance.json`, `ops/release/signing/checksums.json`
- Ops chart/workspace version binding: `ops/release/ops-release-bundle-manifest.json`
- Compatibility output: `docs/_internal/generated/ops-compatibility-matrix.md`
