# Replication Performance Benchmarks

## Benchmark Dimensions

- sync throughput (rows/sec)
- replication lag (ms)
- failover completion latency (ms)

## Example Workloads

- steady write stream at fixed rate
- burst write stream with periodic spikes
- mixed read/write with failover events

## Baseline Recording

Store each run with:

- dataset and shard identifiers
- node topology
- primary/replica count
- lag percentile values
- failover duration values

## Regression Check

Treat any sustained lag increase above 25% from baseline as regression candidate.
