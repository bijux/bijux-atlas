# Non-Goals

Atlas v1/v2 explicitly does not do the following:

1. No write API for mutating published datasets.
2. No implicit "latest" default dataset selection in query endpoints.
3. No raw GFF3/FASTA serving from API paths.
4. No cross-dataset joins or multi-dataset transactional query semantics.
5. No free-text fuzzy search beyond explicitly defined prefix/exact filters.
6. No dynamic ingest in request path.
7. No dependency on external mutable state for deterministic query answers.
8. No hidden fallback that weakens checksum verification guarantees.
9. No guarantee of unrestricted heavy query execution under overload.
10. No requirement for Redis as a hard dependency.
11. No canonical transcript selection policy in v1 (reserved for v2).
