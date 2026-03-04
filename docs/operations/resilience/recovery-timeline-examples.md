# Recovery Timeline Examples

## Example: Node Timeout Recovery

- t+0s: timeout detected
- t+1s: failure event recorded
- t+2s: shard failover executed
- t+3s: replica promotion executed
- t+4s: diagnostics updated

## Example: Partition Recovery

- t+0s: partition injection recorded
- t+1s: chaos run events logged
- t+2s: recovery action started
- t+5s: recovery action completed
