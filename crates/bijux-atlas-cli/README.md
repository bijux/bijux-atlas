# bijux-atlas-cli

`bijux-atlas-cli` owns the end-user `bijux-atlas` executable.

This crate is the binary-owner surface for Atlas command discovery, dataset
inspection, ingest orchestration, validation, and export workflows. Runtime
behavior lives in the `bijux-atlas` library crate; this package owns the
installed executable and the command-line contract around it.
