# bijux-atlas-server

`bijux-atlas-server` owns the long-running `bijux-atlas-server` executable.

This crate is the operator-facing binary-owner surface for Atlas HTTP serving,
runtime configuration loading, telemetry startup, and cache warmup behavior.
The reusable runtime implementation lives in the canonical
`bijux-atlas-runtime` library crate, while `bijux-atlas` remains the
compatibility alias for the historical import path. This package owns the
deployed server process contract.

Public references:

- Project docs: <https://bijux.io/bijux-atlas/>
- Rust API docs: <https://docs.rs/bijux-atlas-server/latest/bijux_atlas_server/>
- Source repository: <https://github.com/bijux/bijux-atlas>
