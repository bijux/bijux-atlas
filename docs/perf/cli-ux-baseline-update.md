---
title: CLI UX Baseline Update Procedure
audience: developer
type: guide
stability: stable
owner: bijux-dev-atlas
last_reviewed: 2026-03-05
tags:
  - perf
  - cli
  - benchmark
---

# CLI UX baseline update procedure

1. Run benchmark in target mode:

```bash
cargo run -p bijux-dev-atlas -- perf cli-ux bench --mode warm-start --format json
```

2. Archive the report as the baseline candidate.
3. Compare candidate against previous baseline:

```bash
cargo run -p bijux-dev-atlas -- perf cli-ux diff <old> <candidate> --format json
```

4. If thresholds regress, document rationale before adopting the new baseline.
5. Commit baseline updates with matching report evidence in the same change set.
