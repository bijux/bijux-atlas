# bijux-atlas-store Architecture

## Responsibility

Artifact/store contract surface: catalog/manifests/artifact path and integrity primitives.

## Boundaries

- No HTTP runtime orchestration.
- No ingest parsing/transformation logic.

## Effects

- IO/FS/Net only through explicit backend interfaces.
- No hidden global runtime behavior.
