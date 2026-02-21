# Control Plane Surface

## Scope

`atlasctl` is the canonical control-plane CLI. User entrypoints must remain stable for documented public commands.

## Command Group Contract

Top-level command groups:

- Stable/public groups: `docs`, `configs`, `dev`, `ops`, `policies`
- Internal group: `internal` (never part of public help/docs unless explicitly requested)

Compatibility aliases are temporary and must be removed during migration PRs.

## Surface Guarantees

- Stable groups keep deterministic help output ordering.
- Internal commands are hidden from public help snapshots.
- JSON-output commands must stay schema-backed and versioned.
- Public docs list only stable groups and supported subcommands.
