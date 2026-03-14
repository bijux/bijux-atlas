# Cargo Deny Notes (Core)

This crate does not maintain a crate-local deny allowlist.

Policy:
- Workspace-level `deny.toml` is authoritative.
- Core must remain dependency-minimal to reduce policy exceptions.
