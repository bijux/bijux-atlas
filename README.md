# bijux-atlas

Deterministic genomics platform with a Rust-native control plane.

## Product Narrative
- Bijux Atlas keeps runtime behavior deterministic and policy enforcement executable.
- Repository governance is enforced by `bijux-dev-atlas` contracts and checks.

## Quick Start
- Run doctor first: `bijux dev atlas check doctor --format text`
- Build control plane: `cargo build -p bijux-dev-atlas`
- Run PR contracts lane: `bijux dev atlas contracts pr --mode static --profile ci`

## Lobby
- Start here: `docs/start-here.md`
- Documentation index: `docs/index.md`
- Contributing policy: `CONTRIBUTING.md`
- Security policy: `SECURITY.md`
- Root contract: `CONTRACT.md`
- Repository map reference: `docs/reference/repo-map.md`

## Documentation Entrypoints
- `docs/index.md`
- `docs/start-here.md`
- `docs/reference/index.md`

## Repository Surfaces
- Control plane contracts and rules are described in `CONTRACT.md`.
- Root shape is described in `ops/inventory/root-surface.json`.
