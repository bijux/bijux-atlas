# Release Model

Owner: `product`  
Type: `concept`  
Reason to exist: define immutable release behavior and supported alias semantics.

## Model

- Releases are immutable once published.
- Dataset identity is explicit: `release/species/assembly`.
- Read aliases may point to a release, but do not change release contents.

## Compatibility

- Existing published releases remain readable within supported compatibility windows.
- Release metadata and checksums are part of the compatibility contract.

## Related Pages

- [Compatibility Promise](compatibility-promise.md)
- [Non Goals](non-goals.md)
- [Release Contract Checklist](release-contract-checklist.md)
