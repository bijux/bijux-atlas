# System Invariant Report Schema

`bijux-dev-atlas invariants run --format json` emits:

- `schema_version` (number)
- `kind` (`system_invariant_report`)
- `status` (`ok` or `failed`)
- `metrics.execution_time_ms` (number)
- `summary.total|failed|passed` (number)
- `registry_completeness.status` (`pass` or `fail`)
- `registry_completeness.missing_in_index[]` (string)
- `registry_completeness.extra_in_index[]` (string)
- `results[]`:
  - `id`
  - `title`
  - `severity`
  - `group`
  - `status`
  - `violations[]`:
    - `class`
    - `message`
    - `path` (nullable)
