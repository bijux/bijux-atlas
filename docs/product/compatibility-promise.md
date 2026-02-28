# Compatibility Promise

Owner: `product`  
Type: `concept`  
Reason to exist: define stable compatibility guarantees for public Atlas surfaces.

## Promise

- Existing documented v1 API paths and fields remain stable.
- Published artifact layouts remain backward-readable within v1.
- Cursor formats remain backward-decodable within v1.
- Existing error code identifiers remain valid within v1.

## Limits

- Undocumented or experimental surfaces are excluded.
- Any intentional break requires explicit versioning policy updates.

## Verification

- `bijux dev atlas contracts check --checks breakage`
- `make openapi-drift`

## Related Pages

- [API Versioning](../api/versioning.md)
- [Errors Reference](../reference/errors.md)
- [What Is Bijux Atlas](what-is-bijux-atlas.md)
