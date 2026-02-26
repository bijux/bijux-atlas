# ARCHITECTURE (bijux-dev-atlas)

- Owner: bijux-dev-atlas
- Stability: stable

This crate-level governance page points to canonical crate docs and root docs.

- Crate docs index: crates/bijux-dev-atlas/docs/INDEX.md
- Central docs index: docs/index.md

## Internal Boundaries (Convergence Target)

- `cli`: argument parsing and command surface definitions.
- `commands`: execution handlers orchestrating adapters + core.
- `core`: pure control-plane logic and checks.
- `ports`: IO traits used by core.
- `adapters`: real IO implementations for ports.
- `model`: serde-facing types and codecs.
- `policies`: policy contracts and validators.

Migration note:
- During crate convergence, code moves behind these modules without changing report/output behavior.
