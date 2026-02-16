# PUBLIC API: bijux-atlas-core

Stable exported items (0.1.x contract):

- `CRATE_NAME`
- `ENV_BIJUX_LOG_LEVEL`
- `ENV_BIJUX_CACHE_DIR`
- `ExitCode`
- `ConfigPathScope`
- `MachineError`
- `ErrorCode`
- `canonical` module
- `time` module
- `sha256_hex`
- `resolve_bijux_cache_dir`
- `resolve_bijux_config_path`

Export policy:
- `lib.rs` must not expose additional public API without updating this file.
