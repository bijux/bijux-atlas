# Pagination Contract

Cursor properties:
- Opaque token = base64(payload) + HMAC signature.
- Payload binds to normalized query hash.
- Order mode is encoded and validated (`gene_id` or `region`).

Stability guarantees:
- Same dataset + same query + same secret => stable cursor sequence.
- Tie-break ordering uses deterministic key fallback (`gene_id`).

Tamper protection:
- Signature mismatch -> stable cursor error.
- Query hash mismatch -> cursor rejected.
