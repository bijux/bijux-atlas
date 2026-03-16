---
title: Structured Output Contracts
audience: mixed
type: contract
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Structured Output Contracts

Structured output contracts define which machine-readable outputs are meant to be stable enough for automation.

## Output Contract Model

```mermaid
flowchart LR
    Command[CLI or API command] --> Json[Structured output]
    Json --> Automation[Automation and CI]
```

## Stability Logic

```mermaid
flowchart TD
    Stable[Documented structured output] --> Parse[Safe to parse]
    Unstable[Undocumented text output] --> Human[Human-only interpretation]
```

## Main Promise

If Atlas documents a structured output surface and tests it, automation should prefer that surface over screen-scraped human text.

## Stability

Only structured outputs that Atlas documents as contracts should be treated as
stable automation inputs. Human-readable text remains descriptive and may
change without compatibility guarantees.

## Purpose

This page defines the Atlas contract expectations for structured output contracts. Use it when you need the explicit compatibility promise rather than a workflow narrative.
