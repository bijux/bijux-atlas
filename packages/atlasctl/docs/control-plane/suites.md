# First-Class Suites

Suite manifests are first-class and deterministic.

- `docs`
- `dev`
- `ops`
- `policies`
- `configs`
- `all`

Each suite manifest defines:

- check IDs
- required environment
- default effects policy
- time budget policy

`atlasctl suite check` treats these manifests as SSOT and fails on drift.
