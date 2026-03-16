---
title: Plugin Contracts
audience: mixed
type: contract
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Plugin Contracts

Plugin contracts define the metadata and compatibility information Atlas exposes for integration-aware consumers.

## Plugin Metadata Model

```mermaid
flowchart LR
    Version[Version endpoint] --> Plugin[Plugin metadata]
    Plugin --> Consumers[Umbrella or integration consumers]
```

## Contract Focus

```mermaid
flowchart TD
    PluginContract[Plugin contract] --> Name[Name]
    PluginContract --> VersionField[Version]
    PluginContract --> Compatibility[Compatible umbrella range]
    PluginContract --> BuildHash[Build hash]
```

## Main Promise

Atlas should expose plugin identity and compatibility metadata in a form that external integrators can reason about without reading internal source code.

## Purpose

This page defines the Atlas contract expectations for plugin contracts. Use it when you need the explicit compatibility promise rather than a workflow narrative.

## Stability

This page is part of the checked-in contract surface. Changes here should stay aligned with tests, generated artifacts, and release evidence.
