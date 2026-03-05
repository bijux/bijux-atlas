---
title: Data Protection Policy Reference
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Data Protection Policy Reference

Policy files:

- `configs/security/data-protection.yaml`
- `configs/security/data-classification.yaml`
- `configs/security/redaction.json`

Runtime linkage:

- enforcement and contracts in `crates/bijux-atlas-core/src/domain/security_data_protection.rs`
- operational telemetry in `crates/bijux-atlas-server/src/telemetry/metrics_endpoint/metrics_runtime/main_handler.rs`
