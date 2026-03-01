# PUBLIC API: bijux-atlas-core

Stability reference: [Stability Levels](../../../docs/_internal/governance/style/stability-levels.md)

Stable exported items (0.1.x contract):

- `CRATE_NAME`
- `ENV_BIJUX_LOG_LEVEL`
- `ENV_BIJUX_CACHE_DIR`
- `NO_RANDOMNESS_POLICY`
- `ExitCode`
- `ConfigPathScope`
- `MachineError`
- `Error`
- `Result<T>`
- `ErrorCode`
- `ERROR_CODES`
- `Hash256`
- `DatasetId`
- `ShardId`
- `RunId`
- `FsPort`
- `ClockPort`
- `NetPort`
- `ProcessPort`
- `ProcessResult`
- `ErrorContext`
- `ResultExt`
- `canonical` module
- `time` module
- `sha256_hex`
- `sha256`
- `no_randomness_policy`
- `resolve_bijux_cache_dir`
- `resolve_bijux_config_path`

Export policy:
- `lib.rs` must not expose additional public API without updating this file.
