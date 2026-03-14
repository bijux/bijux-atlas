# Pagination Contract

Cursor properties:
- Opaque token format (v1) = `v1.<base64(payload)>.<base64(hmac)>`.
- Payload binds to normalized query hash.
- Order mode is encoded and validated (`gene_id` or `region`).
- Decoder is backward-compatible with legacy unversioned `<payload>.<sig>` tokens.
- Cursor payload includes a depth counter; depth above guardrail is rejected.

Stability guarantees:
- Same dataset + same query + same secret => stable cursor sequence.
- Tie-break ordering uses deterministic key fallback (`gene_id`).

Tamper protection:
- Signature mismatch -> stable cursor error.
- Query hash mismatch -> cursor rejected.
