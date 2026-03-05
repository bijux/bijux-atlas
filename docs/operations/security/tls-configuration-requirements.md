---
title: TLS Configuration Requirements
audience: user
type: policy
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# TLS Configuration Requirements

## Required settings

- `transport.tls_required: true` for production-facing deployments
- `transport.min_tls_version` set to approved baseline
- certificate and key paths must resolve to valid PEM material

## Validation surfaces

- `tls_handshake_allowed` in core security data-protection module
- certificate bundle load and validation checks in core
