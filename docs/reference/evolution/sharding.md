# Sharding Evolution: Monolith to Shards

Goal:
- Support large datasets with optional shard layout without breaking API clients.

Compatibility rules:
- `gene_summary.sqlite` remains the stable primary artifact.
- Sharded datasets add `catalog_shards.json` and `gene_summary.<shard>.sqlite` files.
- Request/response contracts are unchanged.

Upgrade path:
1. Produce monolithic + shard artifacts from ingest in the same dataset release.
2. Validate shard index/schema gates (`atlas dataset validate` checks all shards).
3. Enable server shard fan-out for region queries only.
4. Keep non-region queries on monolithic DB unless explicitly optimized later.
5. Roll back by disabling shard fan-out; monolithic artifact remains valid.

Operational guardrails:
- Enforce maximum shard count via policy.
- Enforce max open shards per pod to protect FD/memory limits.
- Keep shard cache eviction independent so cold shards can be dropped first.
