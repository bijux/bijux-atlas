---
title: Telemetry Drills
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Telemetry Drills

Telemetry drills are declared operational rehearsals for gaps in metrics,
logging, and tracing visibility.

## Purpose

Use telemetry drills to verify that Atlas still emits the signals operators
would need during a real outage, dependency loss, or rollout failure.

## Source of Truth

- `ops/observe/drills.json`
- `ops/observe/telemetry-drills.json`
- `ops/observe/drills/drills.json`
- `ops/observe/drills/result.schema.json`
- `ops/observe/suites/suites.json`

## Drill Taxonomy

The drill set currently covers scenarios such as warmup restart, Redis outage,
offline egress denial, catalog unreachability, store failure, offline prewarm
serve, rollout failure recovery, and invalid configuration rejection. The
telemetry drill registry also classifies focused signal drills such as
`observe.drill.otel_outage` and `observe.drill.prometheus_gap`.

## Expected Outputs

Every drill result should record:

- the drill identifier
- start and end times
- pass or fail status
- snapshot paths for metrics, traces, and logs
- any relevant trace identifiers
- the expected signals that were verified

## Success Conditions

A drill succeeds when the documented signals actually appear and remain usable
for diagnosis. A drill fails not only when the system misbehaves, but also when
the expected telemetry is missing, ambiguous, or cannot be correlated.

## Related Contracts and Assets

- `ops/observe/drills/`
- `ops/observe/telemetry-drills.json`
- `ops/observe/drills.json`
- `ops/observe/drills/result.schema.json`
