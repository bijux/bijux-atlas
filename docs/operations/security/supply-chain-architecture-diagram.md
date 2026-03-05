---
title: Supply Chain Architecture Diagram
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Supply Chain Architecture Diagram

```text
[Dependency Sources]
   |  allowlist + lockfiles
   v
[Build And Validation]
   |  vulnerability scan + action pin checks
   v
[Evidence Generation]
   |  SBOM + dependency inventory + audit report
   v
[Release And Monitoring]
   |  compliance report + dashboard artifacts
```
