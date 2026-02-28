# Compatibility Promise

Owner: `product`  
Type: `concept`  
Reason to exist: define stable compatibility guarantees for public Atlas surfaces.

## Promise

- Existing documented v1 API paths and fields remain stable.
- Published artifact layouts remain backward-readable within v1.
- Cursor formats remain backward-decodable within v1.
- Existing error code identifiers remain valid within v1.
- Compatibility changes are additive by default.

## Limits

- Undocumented or experimental surfaces are excluded.
- Any intentional break requires explicit versioning policy updates.

## Verification

- `bijux dev atlas contracts check --checks breakage`
- `make contracts-docs`

## Reproducibility And Stability Guarantees

- Deterministic serialization and stable ordering are required for generated artifacts.
- Tool and configuration inputs must remain pinned in automation.
- Stability commitments apply to documented API, model, and operations contract surfaces.

## Related Pages

- [API Versioning](../api/versioning.md)
- [Errors Reference](../reference/errors.md)
- [What Is Bijux Atlas](what-is-bijux-atlas.md)
