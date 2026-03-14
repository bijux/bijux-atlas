# Versioning

The Python SDK follows semantic versioning.

Compatibility is determined by:

1. Server semantic version compatibility ranges in `compatibility.json`.
2. Runtime discovery via `/version` (fallback `/health`) at client startup or explicit compatibility checks.
3. API surface contract alignment for the `v1` query endpoint family.
