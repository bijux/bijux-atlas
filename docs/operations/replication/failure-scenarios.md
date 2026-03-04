# Replication Failure Scenarios

## Scenario 1: Primary Node Crash

- expected behavior: failover to healthy replica
- verification: primary ownership changes and lag stabilizes

## Scenario 2: Replica Network Isolation

- expected behavior: replica marked degraded
- verification: failure counters increment and diagnostics show reason

## Scenario 3: Replica Storage Errors

- expected behavior: failed checks rise and replica removed from promotion candidates
- verification: health output flags storage failure reason
