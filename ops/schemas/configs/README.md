# Configs Schema Registry

This directory is the control-plane schema registry root for repository config validation.

Rules:

- Schema files here are data-only inputs.
- `bijux dev atlas configs ...` consumes schemas from this tree.
- Do not add executable entrypoints here.
- Keep schema file names intent-based and stable.
