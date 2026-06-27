# bijux-atlas-cli

`bijux-atlas-cli` owns the end-user `bijux-atlas` executable.

This crate is the binary-owner surface for Atlas command discovery, dataset
inspection, ingest orchestration, validation, and export workflows. Runtime
behavior lives in the canonical `bijux-atlas-runtime` library crate, while
`bijux-atlas` remains the compatibility alias for the historical import path.
This package owns the installed executable and the command-line contract around
it.
