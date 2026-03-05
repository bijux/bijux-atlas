# Testing

This crate uses contract-focused tests to lock behavior for CLI output,
generated artifacts, governance policies, and operational boundaries.

Use these commands during local validation:

- `cargo test -p bijux-dev-atlas`
- `cargo test -p bijux-dev-atlas --test cli_smoke`
- `cargo test -p bijux-dev-atlas --test ops_surface_golden`

For the full quality model and escalation rules, see `docs/quality-system.md`.
