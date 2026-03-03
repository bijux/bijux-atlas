# Compatibility test plan

- Owner: `api-contracts`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: define how Atlas verifies that public API guarantees remain compatible within `v1`.

## Coverage

- Stable endpoint paths remain present.
- Stable error codes remain unchanged.
- Cursor semantics remain backward-compatible.
- OpenAPI output remains deterministic for the published `v1` surface.

## Verification commands

```bash
cargo test -q -p bijux-atlas-api openapi_snapshot_is_deterministic_and_matches_committed_contract -- --exact
cargo test -q -p bijux-atlas-api openapi_hash_matches_pinned_contract -- --exact
make contracts
```

Expected output: all checks exit `0` and report no OpenAPI or contract drift.

## Next steps

- [Compatibility](compatibility.md)
- [Reference contracts compatibility](../reference/contracts/compatibility.md)
