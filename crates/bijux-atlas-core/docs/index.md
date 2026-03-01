# Core Docs Index

`bijux-atlas-core` provides deterministic, dependency-light primitives used across atlas crates.

What core is:
- Canonicalization helpers for stable bytes/hashes/cursors.
- Shared machine-readable error types and exit-code contracts.
- Shared config/env-path resolution utilities.

What core must never do:
- No filesystem/network/process effects in core logic.
- No async runtime dependency (`tokio`) or HTTP clients (`reqwest`).
- No wall-clock dependent behavior in deterministic paths.

Documentation map:
- [Architecture](architecture.md)
- [Public API](public-api.md)
- [Effects policy](effects.md)
- [Canonicalization rules](canonicalization.md)
- [Error contract](errors.md)
- [Feature flags policy](features.md)
- [Debug/Display policy](formatting.md)
- [Serialization policy](serde-policy.md)
- [Design patterns](patterns.md)
- [Cargo deny notes](cargo-deny-notes.md)
- [Purity contract](purity.md)

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
