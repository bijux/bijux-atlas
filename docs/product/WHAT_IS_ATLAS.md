# What Is Bijux Atlas

Bijux Atlas is a release-indexed genomic data product.

It has three product surfaces:

1. Builder: deterministic ingest from GFF3+FASTA(+FAI) to validated dataset artifacts.
2. Registry: immutable dataset catalog and manifest contract for discoverability and compatibility.
3. Server: low-latency read API over published datasets with strict policy and observability.

A dataset in Atlas is a product unit, not an ad-hoc file set.
Each dataset is addressed by explicit dimensions:
- `release`
- `species`
- `assembly`

Atlas is optimized for repeatable publication and stable read semantics across versions.
