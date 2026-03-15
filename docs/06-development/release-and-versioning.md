---
title: Release and Versioning
audience: maintainer
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Release and Versioning

Release work is where local correctness becomes public responsibility.

## Release Flow

```mermaid
flowchart TD
    Changes[Validated changes] --> Version[Version and release decisions]
    Version --> Evidence[Compatibility and test evidence]
    Evidence --> Release[Release]
```

## Versioning Model

```mermaid
flowchart LR
    Internal[Internal-only changes] --> Lower[Lower compatibility risk]
    Contract[Contract surface changes] --> Higher[Higher compatibility scrutiny]
```

## Maintainer Priorities

- understand which surfaces changed
- understand whether the change is compatible
- ensure release evidence matches the level of change

## Practical Mindset

Release discipline is not only a packaging step. It is the final check that the documented story, tested story, and shipped story still match.

