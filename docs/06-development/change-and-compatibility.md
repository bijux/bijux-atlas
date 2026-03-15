---
title: Change and Compatibility
audience: maintainer
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Change and Compatibility

Atlas changes should be classified before they are implemented. That prevents accidental breaking changes from sneaking in under the label of simple refactoring.

## Change Classification

```mermaid
flowchart TD
    Change[Change proposal] --> Internal[Internal refactor]
    Change --> Surface[Public surface change]
    Surface --> Compatible[Compatible evolution]
    Surface --> Breaking[Breaking change]
```

## Compatibility Questions

```mermaid
flowchart LR
    Surface[Surface] --> Docs[Documented?]
    Docs --> Tests[Tested?]
    Tests --> Promise[Actually promised?]
```

## Maintainer Checklist

- is this surface documented?
- is it contract-owned?
- do tests enforce the promise?
- does the change alter user, operator, or automation expectations?

## Rule of Thumb

If users, operators, or CI would notice the change without reading source code, treat it as a compatibility question first and an implementation question second.

