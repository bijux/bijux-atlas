---
title: Verify ops artifacts authenticity
owner: bijux-atlas-operations
stability: stable
last_reviewed: 2026-03-05
---

# Verify ops artifacts authenticity

1. Validate publish manifest and digest linkage.

```bash
cargo run -q -p bijux-dev-atlas -- release ops provenance-verify --format json
cargo run -q -p bijux-dev-atlas -- release ops digest-verify --format json
```

2. Validate bundle integrity and required metadata.

```bash
cargo run -q -p bijux-dev-atlas -- release ops bundle-verify --version 0.1.0 --format json
cargo run -q -p bijux-dev-atlas -- release ops scenario-evidence-verify --format json
```

3. Validate release checksums ledger.

```bash
cargo run -q -p bijux-dev-atlas -- release checksums generate --format json
cargo run -q -p bijux-dev-atlas -- release checksums verify --format json
```
