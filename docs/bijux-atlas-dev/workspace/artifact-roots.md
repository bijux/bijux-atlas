---
title: Artifact Roots
audience: maintainers
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Artifact Roots

Atlas keeps generated reports, run products, caches, and examples in distinct
roots because they carry different trust and retention properties. A path under
`artifacts/` is discoverable output, but the path alone does not make it valid
evidence.

```mermaid
flowchart TD
    Run[Named command run] --> Evidence[artifacts/RUN_ID]
    Run --> Isolate[artifacts/isolates/LANE]
    Run --> Cache[.cache or configured Cargo roots]
    Source[Checked-in source] --> Example[ops/_generated.example]
    Evidence --> Retain[Reports, logs, summaries, receipts]
    Isolate --> Dispose[Temporary and lane-scoped execution state]
    Cache --> Dispose
    Example --> Fixture[Reviewed example, not run evidence]
```

## Root Semantics

| Root | Intended contents | Trust rule |
| --- | --- | --- |
| `artifacts/<run-id>/` | reports, logs, summaries, bundles, and captured receipts for one run | bind claims to the run identifier, source revision, command, and target identity |
| `artifacts/isolates/<lane>/` | lane-local temporary directories and, in some workflows, Cargo home or target state | disposable execution state; never cite cache presence as proof a check ran |
| `.cache/` or an explicitly configured Cargo root | compiler and dependency caches | performance optimization only |
| `artifacts/docs/site/` | built documentation site | generated publication input; verify the build and site contract separately |
| `ops/_generated.example/` | checked-in example outputs | fixture or reference material; never present it as a live cluster or release result |

Workflows do not use one universal Cargo cache location. For example,
`ops-validate` places Cargo state beneath its isolate, while
`release-candidate` uses `.cache/cargo/target/release-candidate` and keeps run
reports under `artifacts/<run-id>/`. Consumers must use the paths declared by
the executing lane rather than infer evidence from a generic directory name.

## Run Identity

A reviewable run root contains enough context to answer:

- which command and revision produced the files;
- which profile, selector, or suite was chosen;
- which capabilities and external targets were used;
- which report is authoritative and which files are supporting logs;
- whether the workflow uploaded the complete root and how long it is retained.

The release-candidate and ops-validation workflows create a unique `RUN_ID`,
write reports and logs under that root, summarize key paths, and upload the run
directory. Individual commands can also write domain-specific paths elsewhere
under `artifacts/`; workflow capture must make those paths explicit.

## Repository Boundary

`configs/sources/repository/repo-laws.json` declares runtime artifacts
ephemeral outside governed examples. Generated output is committed only when a
registry or source contract names the governed destination. Otherwise it stays
untracked and disposable.

## Stability

Report paths named by workflows, registries, or consumer automation are
compatibility surfaces. Cache and temporary paths are not, unless a checked-in
consumer incorrectly promotes them into a contract.
