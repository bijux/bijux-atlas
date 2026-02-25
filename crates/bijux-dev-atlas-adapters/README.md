# bijux-dev-atlas-adapters

![Version](https://img.shields.io/badge/version-0.1.0-informational.svg) ![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg) ![Docs](https://img.shields.io/badge/docs-contract-stable-brightgreen.svg)

IO adapter implementations for `bijux-dev-atlas-core` ports.

## Adapter Set
- `RealFs`: filesystem reads/writes with artifacts-root write guard.
- `RealProcessRunner`: subprocess runner using allowlist policy.
- `RealGit`: tracked-file discovery via `git ls-files`.
- `DeniedNetwork`: explicit no-network adapter.
- `RealWorld`: single production bundle.
- `FakeWorld`: deterministic test bundle for mocked behavior.

## Determinism Helpers
- `normalize_line_endings` enforces LF text normalization.
- `sorted_non_empty_lines` provides stable ordering for text inputs.
- `discover_repo_root` resolves repository root with explicit failure modes.

## Safe Extension Rules
- Implement only core port traits in this crate.
- Add new IO capabilities through explicit core ports first.
- Keep default behavior deny-by-default for network and unsafe effects.
