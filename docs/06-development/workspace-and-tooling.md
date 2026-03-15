---
title: Workspace and Tooling
audience: maintainer
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Workspace and Tooling

Atlas lives in a multi-crate workspace. Development works best when you treat the workspace as the unit of truth, not only the single crate you happen to be editing.

## Workspace View

```mermaid
flowchart LR
    Workspace[Workspace root] --> Atlas[crates/bijux-atlas]
    Workspace --> DevAtlas[crates/bijux-dev-atlas]
    Workspace --> Docs[docs/]
    Workspace --> Ops[ops/]
    Workspace --> Configs[configs/]
```

## Tooling View

```mermaid
flowchart TD
    Cargo[Cargo] --> Build[Build and test]
    MkDocs[MkDocs] --> DocsSite[Docs site]
    Make[Make targets] --> Automation[Common workflows]
    DevAtlas[bijux-dev-atlas] --> ControlPlane[Governed checks and reports]
```

## Practical Advice

- run commands from the workspace root
- treat `bijux-dev-atlas` as part of the development toolchain, not as a separate afterthought
- keep artifacts under `artifacts/`
- prefer explicit paths over current-directory assumptions

