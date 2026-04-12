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

This release flow reminds maintainers that versioning is downstream of validated change analysis and
evidence. Release is the public expression of work that was already classified and proven.

## Versioning Model

```mermaid
flowchart LR
    Internal[Internal-only changes] --> Lower[Lower compatibility risk]
    Contract[Contract surface changes] --> Higher[Higher compatibility scrutiny]
```

This versioning model is intentionally simple: the more a change touches contract-owned surfaces, the
more carefully release review should treat it.

## Maintainer Priorities

- understand which surfaces changed
- understand whether the change is compatible
- ensure release evidence matches the level of change

## Release Types

- planned release: normal delivery of accumulated compatible work
- patch release: correctness, regression, or security fixes with narrow scope
- emergency release: urgent mitigation for a high-severity incident or exploit

Each release type still needs explicit evidence. Urgency changes the path length, not the obligation to prove what shipped.

## Support and Deprecation Model

- the latest supported minor line receives normal maintenance
- the previous supported minor line is the fallback window for critical or security fixes
- older lines are unsupported unless the repository explicitly documents an extension

For deprecations:

1. introduce the replacement first
2. record the deprecation in `configs/sources/governance/governance/deprecations.yaml`
3. keep compatibility shims or redirects for the supported window
4. remove the deprecated surface only after the planned removal point and updated evidence

## Practical Governance Checks

Review deprecation entries in `configs/sources/governance/governance/deprecations.yaml` as part of
release preparation, and use this command to inspect the broader governance state:

```bash
cargo run -q -p bijux-dev-atlas -- governance doctor --format json
```

## Practical Mindset

Release discipline is not only a packaging step. It is the final check that the documented story, tested story, and shipped story still match.

## Release Questions Worth Asking

- which user, operator, or automation surfaces changed?
- what evidence proves the release story still matches the docs?
- does the versioning decision reflect the actual compatibility impact?

## Purpose

This page explains the Atlas material for release and versioning and points readers to the canonical checked-in workflow or boundary for this topic.

## Stability

This page is part of the canonical Atlas docs spine. Keep it aligned with the current repository behavior and adjacent contract pages.
