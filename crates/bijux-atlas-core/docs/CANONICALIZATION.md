# Canonicalization Rules

Stable output rules:
- JSON object keys are sorted lexicographically before serialization.
- Array order is preserved as provided by caller.
- Hashing uses SHA-256 over canonical byte representation.
- Cursor payload encoding uses URL-safe base64 without padding.

Determinism constraints:
- Do not include wall-clock timestamps.
- Do not include random values.
- Do not include process-local or host-local metadata.

Ordering helpers:
- `stable_sort_by_key` must be used when order needs explicit reproducibility.
