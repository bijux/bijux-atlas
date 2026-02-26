//! `commands` contains command execution handlers that coordinate CLI, adapters, and core.
//!
//! Boundary: command handlers may orchestrate `adapters` + `core`, but should keep business logic
//! in `core` and parsing concerns in `cli`.
