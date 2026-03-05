---
title: Verify release artifacts
owner: bijux-atlas-operations
stability: stable
last_reviewed: 2026-03-05
---

# Verify release artifacts

```bash
cargo run -q -p bijux-dev-atlas -- release manifest validate --format json
cargo run -q -p bijux-dev-atlas -- release bundle verify --format json
cargo run -q -p bijux-dev-atlas -- release checksums verify --format json
```

Use `release/signing/checksums.json` as the canonical integrity ledger.
