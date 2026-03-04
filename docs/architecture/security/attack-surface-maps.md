---
title: Attack Surface Maps
audience: contributor
type: concept
stability: stable
owner: architecture
last_reviewed: 2026-03-04
tags:
  - architecture
  - security
---

# Attack Surface Maps

Primary externally reachable surfaces:

- HTTP API and headers
- artifact ingestion/download channels
- deployment and workflow automation
- runtime metrics/log export endpoints

Each surface must have explicit controls in auth, authorization, integrity, and monitoring layers.
