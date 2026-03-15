# Ops Error Catalog

Use `bijux-dev-atlas ops validate --format json` to validate the current ops surface.

| Error code | Meaning | Remediation |
| --- | --- | --- |
| `REPO-LAW-001` | Executable scripts detected outside governed exceptions. | Move automation into `bijux-dev-atlas` commands and remove tracked scripts. |
| `REPO-LAW-002` | Root surface drifted outside approved allowlist. | Update root allowlist policy and ownership documentation before adding new root items. |
| `REPO-LAW-003` | Generated or artifact output committed outside approved sinks. | Move outputs under `artifacts/` or approved generated directories, then untrack drift. |
| `REPO-LAW-004` | Inventory metadata or schema coverage drifted from the live tree. | Update the affected `ops/inventory/*.json` or `ops/schema/generated/schema-index.json` entry in the same change. |

Canonical index: `docs/reference/errors-and-exit-codes.md`
