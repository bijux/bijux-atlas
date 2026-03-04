# Failure Injection Guide

## Purpose

Validate detection and recovery behavior in controlled environments.

## Supported Fault Kinds

- `node_crash`
- `shard_corruption`
- `network_partition`

## Command Pattern

POST JSON payload to `/debug/failure-injection`:

```json
{
  "kind": "node_crash",
  "node_id": "node-a"
}
```

## Chaos Run Pattern

POST JSON payload to `/debug/chaos/run` for multi-fault simulation.

## Verification

1. check `/debug/recovery/diagnostics`
2. verify `atlas_failure_events_total` incremented
3. run recovery workflow and confirm recovery metrics
