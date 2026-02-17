# Docs Index

- [Architecture](ARCHITECTURE.md)
- [API Contract](API_CONTRACT.md)
- [Effects](EFFECTS.md)
- [Errors](ERRORS.md)
- [OpenAPI](OPENAPI.md)
- [Patterns](PATTERNS.md)
- [Versioning](VERSIONING.md)
- [Human vs Machine Contracts](HUMAN_MACHINE.md)
- [Public API](PUBLIC_API.md)
- [Public Surface Checklist](PUBLIC_SURFACE_CHECKLIST.md)
- [Tests](../tests/)
- Benches: none

## API stability

Public API is defined only by `docs/PUBLIC_API.md`; all other symbols are internal and may change without notice.

## Invariants

Core invariants for this crate must remain true across releases and tests.

## Failure modes

Failure modes are documented and mapped to stable error handling behavior.

## How to extend

Additions must preserve crate boundaries, update `docs/PUBLIC_API.md`, and add targeted tests.

