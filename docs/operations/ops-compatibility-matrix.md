---
title: Ops Compatibility Matrix
audience: operator
type: reference
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-05
tags:
  - operations
  - compatibility
  - release
related:
  - docs/operations/deploy/installing-ops-product.md
  - docs/_internal/generated/ops-compatibility-matrix.md
---

# Ops Compatibility Matrix

Generate the canonical runtime/chart/client compatibility output with one command:

```bash
bijux-dev-atlas release ops compatibility-matrix --format json
```

To refresh generated docs representations:

```bash
bijux-dev-atlas release ops compatibility-matrix --allow-write --format json
```

Generated outputs:

- `docs/_internal/generated/ops-compatibility-matrix.md`
- `docs/_internal/generated/ops-compatibility-matrix.json`
