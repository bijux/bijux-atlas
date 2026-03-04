---
title: Ops artifact lineage
owner: platform
stability: stable
last_reviewed: 2026-03-05
---

# Ops artifact lineage

```text
ops render/install/validate
  -> artifacts/ops/evidence/<run_id>/*.json
  -> release/evidence/manifest.json
  -> release/evidence/bundle.tar
  -> ops evidence verify / ops evidence diff
```

This lineage is the canonical audit path for operational readiness evidence.
