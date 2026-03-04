# Dependency Version Monitoring

Atlas tracks version drift by generating dependency inventory from lockfiles during security validation.

Signals tracked:

- Rust package set from `Cargo.lock`
- npm package set from lockfiles
- Python package set from lockfiles
- Helm chart dependency set from lockfiles

Artifact:

- `artifacts/security/dependency-inventory.json`

Use this artifact to compare changes between release candidates and detect unexpected dependency movement.
