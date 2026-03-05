---
title: Reproducible build posture
owner: bijux-atlas-operations
stability: stable
last_reviewed: 2026-03-05
---

# Reproducible build posture

Guaranteed:
- deterministic release manifest generation
- deterministic bundle member ordering
- checksums ledger for shipped artifacts

Not guaranteed:
- third-party registry retention behavior
- cross-host image digest parity without controlled build environment

Primary verification command:

```bash
cargo run -q -p bijux-dev-atlas -- release reproducibility report --format json
```
