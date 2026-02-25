# Contract

- Area: `ops/k8s`
- schema_version: `1`

Canonical parent contract: `ops/CONTRACT.md`.

## Invariants

- Helm chart remains specification-only content.
- Install matrix profile names are unique and lexicographically sorted.
- Canonical values profiles include `kind`, `dev`, `ci`, and `prod`.
- Render artifact index and release snapshot are generated and deterministic.
