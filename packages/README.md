# packages

This directory contains package implementations grouped by language/runtime and product surface.

- `atlasctl`: internal Python tooling package and CLI for repository automation.

Placement rule:
- New implementations in Python or other languages must be added under `packages/` as dedicated package/module directories.
- Keep each package README explicit about ownership, purpose, and boundaries.

Repository policy: no new business logic may be added under `scripts/`; new scripting logic must be implemented via `atlasctl`.
