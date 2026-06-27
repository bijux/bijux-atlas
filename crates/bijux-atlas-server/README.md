# bijux-atlas-server

`bijux-atlas-server` owns the long-running `bijux-atlas-server` executable.

This crate is the operator-facing binary-owner surface for Atlas HTTP serving,
runtime configuration loading, telemetry startup, and cache warmup behavior.
The reusable runtime implementation lives in the `bijux-atlas` library crate;
this package owns the deployed server process contract.
