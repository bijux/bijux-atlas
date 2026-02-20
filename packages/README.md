# packages

This directory contains Python package surfaces with clear role boundaries.

- `atlasctl`: internal tooling package and CLI for repository automation.
- `bijux-atlas-py`: scaffold for future user-facing Python library API.

Boundary rule: user-facing library code must not depend on internal tooling package code.

Repository policy: no new business logic may be added under `scripts/`; new scripting logic must be implemented via `atlasctl`.
