---
title: Artifact Lifecycle
audience: mixed
type: concept
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-12
---

# Artifact Lifecycle

Atlas artifacts move through a predictable lifecycle:

- build from validated inputs
- verify the produced shape
- publish into a serving store
- expose through catalog and runtime lookup
- compare or retire through release-aware workflows

## Why This Matters

The lifecycle is the hinge between ingest work and serving work. If artifact
state is unclear, both runtime behavior and operations drift.
