# Recovery Workflow

## Automatic Recovery

The recovery runner performs:

1. node timeout detection
2. shard ownership failover from timed-out nodes
3. replica primary failover for affected shard groups
4. recovery event recording

## Trigger Endpoint

- `POST /debug/recovery/run`

## Diagnostics Endpoint

- `GET /debug/recovery/diagnostics`

## Recovery Signals

- `timed_out_nodes`
- `shard_failovers`
- `replica_failovers`
- recovery event history
