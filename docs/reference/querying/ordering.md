# Ordering

- Owner: `bijux-atlas-query`

Default ordering is deterministic and query-class specific.

- Gene list defaults to `(seqid, start, gene_id)`.
- Tie-break keys are always explicit and stable.
- Ordering never depends on insertion order.
