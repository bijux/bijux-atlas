# bin Policy

Root `bin/` contains only tiny command shims.

Canonical location for legacy atlasctl wrapper is:
- `packages/atlasctl/bin/atlasctl`

Compatibility shim remains at:
- `./bin/atlasctl`

Rules:
- no business logic
- no network or file mutation logic
- each shim delegates directly to canonical package commands
- keep each shim <= 30 lines
