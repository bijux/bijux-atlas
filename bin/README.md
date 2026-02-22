# bin Policy

Root `bin/` contains only tiny command shims.

Canonical supported local entrypoint for atlasctl is:
- `./bin/atlasctl`

Rules:
- no business logic
- no network or file mutation logic
- each shim delegates directly to canonical package commands
- keep each shim <= 30 lines
