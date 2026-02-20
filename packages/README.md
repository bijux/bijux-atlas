# packages

This directory contains Python package surfaces with clear role boundaries.

- `bijux-atlas-scripts`: internal tooling package and CLI for repository automation.
- `bijux-atlas-py`: scaffold for future user-facing Python library API.

Boundary rule: user-facing library code must not depend on internal tooling package code.
