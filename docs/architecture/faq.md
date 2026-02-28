# Architecture FAQ

Owner: `architecture`  
Type: `concept`  
Reason to exist: replace scattered architecture clarifications with one canonical page.

## Why are published artifacts immutable?

Immutability prevents silent data drift and keeps query reproducibility stable.

## Why can’t query and API layers patch data?

Read-path mutation breaks deterministic behavior and hides source-of-truth defects.

## Why are cross-layer fixups forbidden?

Cross-layer fixes obscure ownership, delay root-cause correction, and create non-repeatable outages.

## Related Pages

- [Architecture](index.md)
- [Storage](storage.md)
- [Performance Model](performance-model.md)
