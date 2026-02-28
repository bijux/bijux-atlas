# Capabilities model

- Owner: `platform`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@331751e4`
- Reason to exist: define subprocess, network, and filesystem capability boundaries.

## Capability types

- `subprocess`: invokes external tools (`helm`, `kubectl`, scanners, generators).
- `network`: allows remote calls when required by a specific check.
- `fs_write`: allows writes for generated outputs and evidence artifacts.

## Examples

- `make docs-build` expands to `docs build --allow-subprocess --allow-write`, so it needs `subprocess` and `fs_write`.
- `make docs-serve` expands to `docs serve --allow-subprocess --allow-network`, so it adds `network`.
- `check run --suite ci_pr` is normally static unless a selected check explicitly requires a capability.

## Rules

- Default behavior is least privilege.
- Required capabilities are explicit in command invocation.
- Lane policy restricts where higher-risk capabilities are allowed.

## Verify success

```bash
cargo run -q -p bijux-dev-atlas -- docs inventory --help
```

## Next steps

- [Static and effect mode](static-and-effect-mode.md)
- [Tooling dependencies](tooling-dependencies.md)
- [Why a control-plane](why-a-control-plane.md)
