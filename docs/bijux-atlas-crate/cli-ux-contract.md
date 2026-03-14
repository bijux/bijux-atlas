# CLI UX Contract

## Command Shape

The stable product surface is noun-first:
- `bijux-atlas dataset <verb>`
- `bijux-atlas catalog <verb>`
- `bijux-atlas diff <verb>`
- `bijux-atlas policy <verb>`
- `bijux-atlas ingest <verb>`

Compatibility shim:
- `bijux-atlas atlas <...>` is accepted as a hidden legacy alias.

## Output Modes

- Default mode is human-readable text.
- `--json` emits canonical deterministic JSON suitable for CI snapshots.

## Stable Flags

- Global: `--json`, `--quiet`, `--verbose`, `--trace`.
- Compatibility metadata: `--umbrella-version`, `--bijux-plugin-metadata`, `--print-config-paths`.

## Error Contract

- Usage and validation failures return `exit code 2`.
- `--json` errors return a machine payload with stable `code` values.

## Non-goals

- Dev-ops orchestration commands are not part of stable end-user CLI surface.
