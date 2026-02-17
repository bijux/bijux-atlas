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
- [Architecture](ARCHITECTURE.md)
- [Public API](PUBLIC_API.md)
- [Effects policy](EFFECTS.md)
- [Canonicalization rules](CANONICALIZATION.md)
- [Error contract](ERRORS.md)
- [Feature flags policy](FEATURES.md)
- [Debug/Display policy](FORMATTING.md)
- [Serialization policy](SERDE_POLICY.md)
- [Design patterns](PATTERNS.md)
- [Cargo deny notes](CARGO_DENY_NOTES.md)
- [Purity contract](PURITY.md)

## API stability

Public API is defined only by `docs/PUBLIC_API.md`; all other symbols are internal and may change without notice.

## Invariants

Core invariants for this crate must remain true across releases and tests.

## Failure modes

Failure modes are documented and mapped to stable error handling behavior.

## How to extend

Additions must preserve crate boundaries, update `docs/PUBLIC_API.md`, and add targeted tests.

