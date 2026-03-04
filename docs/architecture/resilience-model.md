# Resilience Model

## Failure Handling Philosophy

Atlas treats failure as a normal runtime condition and requires deterministic, observable recovery behavior.

## Failure Categories

- node unreachable
- shard corruption
- replica lag
- network partition
- resource exhaustion
- unknown

## Failure Detection Mechanisms

- membership heartbeat timeout detection
- replication lag threshold checks
- explicit fault injection for operator validation

## Recovery Policies

- automatic recovery workflow enabled by default
- shard ownership failover enabled
- replica primary failover enabled
- post-recovery rebalancing enabled

## Resilience Guarantees

- bounded failover objective (`failover_within_ms`)
- diagnostics availability through dedicated endpoint
- recovery and failure event logging as a contract

## Runtime Endpoints

- `POST /debug/recovery/run`
- `GET /debug/recovery/diagnostics`
- `POST /debug/failure-injection`
- `POST /debug/chaos/run`

## Resilience Metrics

- `atlas_failure_events_total`
- `atlas_recovery_events_total`
- `atlas_recovery_success_total`
- `atlas_recovery_failed_total`
- `atlas_recovery_latency_avg_ms`
