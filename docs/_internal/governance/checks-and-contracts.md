---
title: Checks And Contracts
audience: maintainer
type: reference
status: canonical
owner: atlas-governance
last_reviewed: 2026-03-15
---

# Checks And Contracts

## Checks

Checks validate repository state and should be idempotent when rerun against the same inputs.
They are the default surface for policy, docs, config, and lane validation.
Use `check run` when the caller needs one governed suite, one check id, or one tag selection.

## Contracts

Contracts validate long-lived boundary promises between producers and consumers.
They run through the `contracts` suite so the same inventory can be selected in pure, effect, or full execution modes.
Use contract entries when the boundary itself is the governed object, not just a repository hygiene rule.

## Pure And Effect

Pure execution reads state and emits evidence without mutating the environment.
Effect execution may write artifacts, call subprocesses, or interact with live substrates.
The caller must declare those capabilities explicitly so local runs and CI runs stay equivalent.

## Suite Boundaries

The `checks` suite owns repository validation and lane composition.
The `contracts` suite owns boundary verification and contract evidence.
An entry belongs to one suite by default; overlap must be deliberate, documented, and rare.

## Validation System

| Surface | Command family | Selection model | Capability model |
| --- | --- | --- | --- |
| checks | `check run` | suite, id, group, tag | static or effect per check |
| contracts | `suites run --suite contracts` | suite, group, tag, mode | pure, effect, or all |
| make wrappers | `make <target>` | curated public entrypoints | delegated to `bijux-dev-atlas` |
