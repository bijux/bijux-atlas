# Compatibility With bijux-dna

`bijux-atlas-model` has no dependency on `bijux-dna` crates.

Compatibility boundary is artifact contract only:
- dataset identifiers
- manifest/catalog schema
- deterministic checksums and stats

Any bijux-dna integration must happen through artifact production/consumption adapters, never crate-level dependencies.
