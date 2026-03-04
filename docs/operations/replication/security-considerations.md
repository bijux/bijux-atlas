# Replication Security Considerations

## Transport

- use authenticated and encrypted links between primary and replicas.
- reject replication traffic from unknown node identities.

## Access Control

- restrict failover actions to administrative roles.
- audit every promotion event and diagnostics access.

## Data Exposure

- keep replica diagnostics free from sensitive payload content.
- include identifiers and counters only; avoid raw record values.
