# Config Discovery

Resolution order:

1. Explicit command flags.
2. Environment variables (`BIJUX_LOG_LEVEL`, `BIJUX_CACHE_DIR`).
3. Workspace config path from the `bijux_atlas::core` resolver.
4. User config path from the `bijux_atlas::core` resolver.

Use `--print-config-paths` to inspect resolved paths.
