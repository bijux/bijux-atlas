# Store Docs Index

Responsibilities:
- Enforce artifact publish/read contract at storage boundary.
- Provide local and remote backends with stable error mapping.
- Keep atomic publish and immutability guarantees explicit.

Strict boundaries:
- Store must not depend on API/server frameworks.
- Store owns storage effects only (filesystem/network), not query execution.

Docs:
- [Architecture](architecture.md)
- [Public API](public-api.md)
- [Artifact contract](artifact-contract.md)
- [Effects policy](effects.md)
- [Caching semantics](caching.md)
- [Backends and guarantees](backends-and-guarantees.md)
- [Failure modes](failure-modes.md)
- [Rollback workflow](internal/rollback.md)
- [Store outage runbook snippet](internal/runbook-snippet.md)

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
