---
title: Scenario Registry
audience: operators
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Scenario Registry

The load scenario registry keeps workload coverage explicit across generated and
hand-authored scenario files.

## Purpose

Use the scenario registry to understand where Atlas load scenario identity comes
from and how coverage is tracked across authored and generated inputs.

## Source of Truth

- `ops/load/scenario-registry.json`
- `ops/load/scenarios/`
- `ops/load/generated/concurrency-stress-scenarios.json`
- `ops/load/suites/suites.json`

## Registry Semantics

`ops/load/scenario-registry.json` is intentionally small but important. It
declares the registry inputs that together define the Atlas load surface:

- `ops/load/scenarios/core-capacity-scenarios.json`
- `ops/load/suites/suites.json`
- `ops/load/generated/concurrency-stress-scenarios.json`

This means scenario identity is distributed across:

- authored scenario files under `ops/load/scenarios/`
- the suite registry that binds those scenarios to operational lanes
- generated scenario expansions that add structured coverage, such as the
  concurrency stress catalog

## Coverage Expectations

Every scenario used for promotion or regression review should be:

- named in the authored or generated registry inputs
- reachable from a suite or validation lane
- backed by thresholds and expected metrics
- understandable by a later operator without reading generator code first

## Related Contracts and Assets

- `ops/load/scenario-registry.json`
- `ops/load/scenarios/`
- `ops/load/generated/concurrency-stress-scenarios.json`
- `ops/load/suites/suites.json`
