# ADR-0002: SQLite As Canonical Serving Store

Status: Accepted

Context:
Atlas needs deterministic, portable artifacts for read-heavy genomic query serving.

Decision:
- Use SQLite gene/transcript summary artifacts as canonical read store.
- Build artifacts deterministically in ingest.
- Validate required indexes before publish/serve.

Consequences:
- Strong local/dev portability.
- Requires explicit tuning and query-plan regression tests.
