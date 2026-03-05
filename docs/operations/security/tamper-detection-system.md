---
title: Tamper Detection System
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Tamper Detection System

Tamper detection is implemented by `detect_tampering` and runtime integrity checks.

## Signals

- `integrity.violation`
- `tamper.detected`

These signals are exported to metrics and operational alerts.
