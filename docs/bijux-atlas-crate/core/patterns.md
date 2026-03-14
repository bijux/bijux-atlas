# Design Patterns Used

- Functional core: pure transformation helpers are preferred.
- Explicit contracts: stable schemas, stable sorting, stable hashing.
- Narrow exports: only intentionally stable public items are re-exported.
- Deterministic data structures for machine-facing output (`BTreeMap`).
- Forward-compatibility: public enums are `#[non_exhaustive]`; avoid trait surfaces that require sealing until needed.
