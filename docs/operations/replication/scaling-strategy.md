# Replication Scaling Strategy

## Horizontal Scaling

- increase replica count for read-heavy shards.
- colocate replicas across failure domains to reduce correlated outages.

## Throughput Scaling

- tune sync concurrency for high ingest workloads.
- separate heavy query traffic from promotion candidates.

## Capacity Planning

Track per shard:

- average lag
- sync throughput
- promotion frequency
- failure count

Scale replica groups when lag grows under sustained normal load.
