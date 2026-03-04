# Consistency Contract

## Default Contract

- read consistency: `quorum`
- write consistency: `quorum`

## Interpretation

- reads require agreement across quorum replicas when available.
- writes are acknowledged after quorum durability.
- failover preserves quorum semantics for the next elected primary.

## Contract Data

Consistency is exposed in diagnostics output alongside policy:

- read consistency level
- write consistency level
- replication factor
- lag budget
