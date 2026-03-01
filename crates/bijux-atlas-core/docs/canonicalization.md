# Canonicalization Rules

Stable output rules:
- JSON object keys are sorted lexicographically before serialization.
- Array order is preserved as provided by caller.
- Canonical JSON representation is produced through `CanonicalJson`.
- Hashing uses SHA-256 over canonical byte representation.
- Hash values are represented by `Hash256` instead of raw byte arrays in public API.
- `Hash256` avoids exposing raw `[u8; 32]` in public signatures.
- Signed-zero float values are normalized (`-0.0` -> `0`) before canonical serialization.
- Cursor payload encoding uses URL-safe base64 without padding.

Determinism constraints:
- Do not include wall-clock timestamps.
- Do not include random values.
- Do not include process-local or host-local metadata.

Ordering helpers:
- `stable_sort_by_key` must be used when order needs explicit reproducibility.
