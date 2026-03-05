---
title: Mitigation Strategy
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Mitigation Strategy

Mitigation strategy follows layered controls:

1. Preventive controls: auth boundaries, schema validation, strict defaults.
2. Detective controls: audit logs, threat verification, integrity checks.
3. Corrective controls: runbooks, incident workflows, rollback procedures.

Each threat entry must map to at least one mitigation ID in `mitigations.yaml`.
