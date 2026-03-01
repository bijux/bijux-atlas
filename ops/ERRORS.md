# Ops Error Catalog

Use `bijux dev atlas contracts root --mode static` to validate and remediate repo-law violations.

| Error code | Meaning | Remediation |
| --- | --- | --- |
| `REPO-LAW-001` | Executable scripts detected outside governed exceptions. | Move automation into `bijux-dev-atlas` commands and remove tracked scripts. |
| `REPO-LAW-002` | Root surface drifted outside approved allowlist. | Update root allowlist policy and ownership documentation before adding new root items. |
| `REPO-LAW-003` | Generated or artifact output committed outside approved sinks. | Move outputs under `artifacts/` or approved generated directories, then untrack drift. |
| `REPO-LAW-004` | Contract metadata missing required identifiers or ownership fields. | Add contract ID, severity, and owner metadata in registry and mapped docs. |
