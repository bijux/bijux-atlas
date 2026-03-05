---
title: Encryption Architecture Diagram
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Encryption Architecture Diagram

```text
[Ingress TLS Boundary]
        |
        v
[Atlas Runtime]
  |  checksum verify
  |  signature verify
  |  tamper detect
  v
[Artifact Store + Manifest]
  |  provenance metadata
  v
[Evidence And Monitoring]
```
