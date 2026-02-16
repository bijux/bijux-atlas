# Strict Ordering Rules

Stable ordering keys:
- Datasets: `(release, species, assembly)` lexical order.
- Catalog entries: strict sorted order by full `CatalogEntry` ordering.
- Genes (model guidance for query/ingest): `(seqid, start, gene_id)` unless query-specific override is explicitly documented.
