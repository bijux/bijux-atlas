# Config Discovery

Resolution order:

1. Explicit command flags.
2. Environment variables (`BIJUX_LOG_LEVEL`, `BIJUX_CACHE_DIR`).
3. Workspace config path from `bijux_atlas::runtime::config::resolve_bijux_config_path`.
4. User config path from `bijux_atlas::runtime::config::resolve_bijux_config_path`.

Use `--print-config-paths` to inspect resolved paths.
