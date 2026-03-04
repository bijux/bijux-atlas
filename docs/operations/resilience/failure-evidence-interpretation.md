---
title: Interpret failure evidence bundles
audience: operators
type: guide
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags: [resilience, evidence]
---

# Interpret failure evidence bundles

- `failure-classification.json`: class, expected behavior, and recommended action.
- `metrics-snapshot.json`: failure-adjacent counters and latency/error signals.
- `config-snapshot.json`: runtime configuration state at evidence time.
- `logs-snapshot.txt`: operator-facing log summary for rapid triage.
- `result.json` and `summary.md`: scenario-level deterministic report envelope.
