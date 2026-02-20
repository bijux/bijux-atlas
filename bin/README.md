# bin Policy

Root `bin/` contains only tiny command shims.

Rules:
- no business logic
- no network or file mutation logic
- each shim delegates directly to canonical package commands
- keep each shim <= 30 lines
