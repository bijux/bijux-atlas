# Make Internal Contracts

This directory documents enforcement rules for the make surface.

- Make targets are entrypoints only.
- Command orchestration belongs in `bijux-dev-atlas` Rust commands.
- Public make targets delegate to canonical wrappers and avoid ad hoc pipelines.
