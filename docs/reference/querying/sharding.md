# Sharding Behavior

- Owner: `bijux-atlas-query`

In sharded datasets, region queries fan out only to relevant shards with bounded concurrency.

- Non-region lookups use global/indexed shard paths.
- Response semantics are identical between monolithic and sharded modes.
