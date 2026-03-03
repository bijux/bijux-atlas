# Module Map

This index names the primary `bijux-dev-atlas` modules and their stable responsibilities.

| Module | Responsibility |
| --- | --- |
| `engine` | Runnable execution, selection, reporting, and effect gating |
| `model` | Domain-neutral types, report headers, ids, and history models |
| `registry` | Registry loading, validation, indexing, and report catalogs |
| `runtime` | Filesystem, process, world, and artifact boundary adapters |
| `domains` | Domain plugins for checks, contracts, runtime surfaces, and docs links |
| `adapters` | External command-surface metadata and adapter-owned routing surfaces |
| `ui` | Human-readable terminal rendering and presentation helpers |
| `commands` | Legacy orchestration entrypoints still being narrowed behind adapters |
| `contracts` | Contract catalogs and contract-owned validation coverage |
| `core` | Existing business logic that remains under migration into engine/registry/domains |
