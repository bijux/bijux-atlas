# Docs Index

- [Architecture](architecture.md)
- [API Contract](API_CONTRACT.md)
- [Effects](effects.md)
- [Errors](ERRORS.md)
- [OpenAPI](OPENAPI.md)
- [Patterns](PATTERNS.md)
- [Versioning](VERSIONING.md)
- [API Stability and Versioning](API_STABILITY.md)
- [Human vs Machine Contracts](wire-compatibility.md)
- [Public API](public-api.md)
- [Public Surface Checklist](public-api-checklist.md)
- [Tests](../tests/)
- [Benches](../benches/)

- [How to test](testing.md)
- [How to extend](#how-to-extend)

## API stability

Public API is defined only by `docs/public-api.md`; all other symbols are internal and may change without notice.

## Invariants

Core invariants for this crate must remain true across releases and tests.

## Failure modes

Failure modes are documented and mapped to stable error handling behavior.

## How to extend

Additions must preserve crate boundaries, update `docs/public-api.md`, and add targeted tests.

- Central docs index: docs/index.md
- Crate README: ../README.md
