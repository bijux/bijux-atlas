---
title: Release Model
audience: mixed
type: concept
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-12
---

# Release Model

Releases are the time-shaped contract boundary for Atlas dataset content.

Atlas does not treat the latest local build as the durable truth. It treats a
release as the named state that can be validated, published, queried, compared,
and rolled back deliberately.

## Release Questions

- what content belongs to this versioned state
- which artifact set represents it
- how clients request it
- how operators compare or restore it

## Practical Effect

Release-shaped thinking keeps runtime behavior and publication discipline tied
to the same identity model.
