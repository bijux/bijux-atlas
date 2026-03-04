---
title: Evidence Viewer
audience: operator
type: guide
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-03
---

# Evidence Viewer

- Owner: `bijux-atlas-operations`
- Type: `guide`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Last changed: `2026-03-03`
- Reason to exist: show the fastest way to inspect a generated evidence package.

## Fast Path

1. Open `release/evidence/index.html` for the high-level summary.
2. Read `release/evidence/manifest.json` for the canonical file list and checksums.
3. Read `release/evidence/identity.json` for release identity and governance linkage.
4. Use `ops evidence diff` when comparing two bundles.

## Quick Commands

```bash
open release/evidence/index.html
cat release/evidence/manifest.json
cargo run -q -p bijux-dev-atlas -- ops evidence diff release/evidence/bundle.tar other-bundle.tar --allow-write --format json
```
