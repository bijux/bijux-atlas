# scripts/ Deprecation Window

This directory is deprecated.

Policy:
- No new logic is allowed under `scripts/`.
- New tooling must be implemented in `packages/bijux-atlas-scripts/`.
- Public entrypoints must go through `bin/bijux-atlas`.

Migration tracking:
- Run `bin/bijux-atlas legacy audit` to list remaining `scripts/` references.
- Run `bin/bijux-atlas legacy check` to enforce the tracked allowlist in `configs/layout/scripts-references-allowlist.txt`.

Removal plan:
- Migrate each remaining reference to package commands or `ops/run/*` wrappers.
- Delete `scripts/` once `legacy check` reports zero tracked references and all workflows are updated.
