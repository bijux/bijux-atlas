# No DNA Dependency Policy For Atlas

`bijux-atlas` must not depend on `bijux-dna` crates.

Enforcement:

- policy guardrail test scans `Cargo.toml` files and fails if `bijux-dna` is referenced.
- integration remains contract-level only (artifact contracts / API contracts), not crate-level coupling.
