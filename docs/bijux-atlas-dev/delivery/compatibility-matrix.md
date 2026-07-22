---
title: Compatibility Matrix
audience: maintainers
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Compatibility Matrix

Atlas treats compatibility as a set of independently versioned contracts. An
environment variable can remain compatible while a report schema breaks; a
documentation move can be safe only when its former URL redirects. The
compatibility registry makes those distinctions reviewable.

```mermaid
flowchart LR
    Change[Proposed change] --> Classify{Contract surface}
    Classify --> Runtime[Environment, chart, or profile key]
    Classify --> Evidence[Report schema or check identifier]
    Classify --> Reader[Documentation URL]
    Runtime --> Rule[Apply surface rule]
    Evidence --> Rule
    Reader --> Rule
    Rule --> Proof[Alias, registry entry, redirect, or compatibility note]
    Proof --> Window[Retain proof for the deprecation window]
```

The authority is
`configs/sources/governance/governance/compatibility.yaml`. It defines what is
breaking, what a rename must preserve, and how long an overlap remains in
force. It does not infer compatibility from version numbers or release notes.

## Governed Surfaces

| Surface | Breaking examples | Required rename evidence | Window |
| --- | --- | --- | --- |
| environment keys | removal, requiredness change, rename without overlap | allowlist coverage, registry entry, documentation | 180 days |
| chart values | removal, type change, safety-default change | old and new keys accepted, warning for the old key | 180 days |
| profile keys | removal of a consumed key or premature alias removal | registry entry and warning-report coverage | 180 days |
| report schemas | required-field removal, type change, identity change without notice | registry entry and compatibility note | 180 days |
| check identifiers | removal without replacement or archival record | registry entry and overlapping runnable identifiers | 180 days |
| documentation URLs | move without redirect or reuse with different meaning | redirect entry and documentation update | 365 days |

## Review a Change

1. Classify every externally observed surface changed by the patch.
2. Compare the change with the breaking examples for that surface.
3. Add the required alias, warning, registry entry, compatibility note, or
   redirect before removing the old behavior.
4. Record the removal target and keep both forms usable for the declared
   window.
5. Validate the generated or runtime evidence that proves the overlap exists.

A change that touches several rows carries several obligations. Preserving an
old environment key, for example, does not compensate for renaming a check
identifier without an overlap. The matrix is a classification and retention
contract; the owning implementation and its validation evidence prove that the
contract was actually honored.
