# Ordering Rules

Deterministic tie-break policy for v1 gene queries:

1. Region queries: `(seqid ASC, start ASC, gene_id ASC)`.
2. Non-region queries: `(gene_id ASC)`.
3. When two rows share the same coordinates, `gene_id` is the final stable tie-breaker.

This ordering is part of the pagination contract and must not change in v1.
