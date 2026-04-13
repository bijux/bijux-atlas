---
title: Issue Templates
audience: maintainers
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Issue Templates

Issue templates capture recurring maintainer inputs such as performance
regressions and security advisory workflows.

## Intake Model

```mermaid
flowchart TD
    Issue[New issue] --> Choose[Choose template]
    Choose --> Bug[Bug]
    Choose --> Feature[Feature or change request]
    Choose --> Governance[Governance or docs issue]
    Choose --> Release[Release or workflow issue]

    Bug --> Intake[Structured intake]
    Feature --> Intake
    Governance --> Intake
    Release --> Intake

    Intake --> Routing[Better triage and routing]
```

This diagram matters because issue templates are really intake contracts. They
decide what information a maintainer gets before triage and whether the issue
arrives with enough structure to route cleanly.

## Repository Anchors

- [`.github/ISSUE_TEMPLATE/perf-regression.md`](/Users/bijan/bijux/bijux-atlas/.github/ISSUE_TEMPLATE/perf-regression.md:1)
- [`.github/ISSUE_TEMPLATE/security-advisory.yml`](/Users/bijan/bijux/bijux-atlas/.github/ISSUE_TEMPLATE/security-advisory.yml:1)

## Main Takeaway

Issue templates are part of Atlas workflow ownership because they shape the
quality of maintainer intake before any review or validation begins. A good
template reduces triage guesswork and makes routing more deliberate.
