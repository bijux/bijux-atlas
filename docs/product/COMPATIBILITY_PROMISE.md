# Compatibility Promise

Atlas compatibility promises apply to two contracts:

1. API contract (`/v1/*`):
- Additive evolution only inside v1 unless explicitly documented.
- Existing fields and machine error codes remain stable.
- Pagination cursor validation remains strict and deterministic.

2. Artifact contract (`manifest.json`, SQLite schema, catalog linkage):
- Published dataset artifacts are immutable.
- Schema version is explicit and validated.
- Unknown fields in strict contracts are rejected where documented.

Compatibility boundaries:
- Breaking changes require new versioned surface (`/v2` or new artifact schema version).
- Bijux umbrella/plugin compatibility is declared by metadata handshake.
