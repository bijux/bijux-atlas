# Dataset Locality And Shard-Aware Scaling

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

## Node affinity recommendation (dataset locality)

Use node affinity to keep Atlas pods close to node-local cache or SSD volumes.

Recommended labels:
- `bijux.io/atlas-cache-node=true`
- `topology.kubernetes.io/zone=<zone>`

Recommended scheduling behavior:
- Prefer nodes that already host warm Atlas cache volumes.
- Keep pods in the same zone as object-store gateway when possible.

## Shard-aware pod scaling

For sharded datasets (`catalog_shards.json` present):
- Increase pod count when shard fan-out dominates query latency.
- Keep `ATLAS_MAX_OPEN_SHARDS_PER_POD` bounded to protect file descriptors.
- Tune HPA with heavy-query in-flight metrics, not CPU alone.
- Use `make ops-warm-shards` to prewarm common contig shards after deploy.

## Optional consistent hashing routing

Use rendezvous hashing to route a dataset key to a preferred pod:
- key: `release/species/assembly`
- input nodes: active pod identities
- output: one preferred pod for cache locality

This is optional and should degrade to standard load balancing if membership is unstable.

## Cross-pod warm coordination

Optional Redis lock-based warmup coordination is available:
- `ATLAS_WARM_COORDINATION_ENABLED=true`
- `ATLAS_WARM_COORDINATION_LOCK_TTL_SECS=<seconds>`
- requires `ATLAS_REDIS_URL`

Purpose: reduce startup stampede where many pods warm the same dataset simultaneously.
## Referenced chart values keys

- `values.affinity`
- `values.nodeSelector`
- `values.cache`

## See also

- `ops-ci`
