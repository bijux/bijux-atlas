# bijux-atlas

Deterministic genomics platform with a Rust-native control plane.

## Product Narrative
- Bijux Atlas keeps runtime behavior deterministic and policy enforcement executable.
- Repository governance is enforced by `bijux-dev-atlas` contracts and checks.

## Quick Start
- Build control plane: `cargo build -p bijux-dev-atlas`
- Run complete contracts lane: `bijux dev atlas contracts all --mode effect --profile ci --allow-subprocess --allow-network --allow-k8s --allow-fs-write --allow-docker-daemon`
- Start docs onboarding: `docs/start-here.md`

## Documentation Entrypoints
- Home: `docs/index.md`
- Start here: `docs/start-here.md`
- API: `docs/api/index.md`
- Operations: `docs/operations/index.md`
- Development: `docs/development/index.md`
- Reference: `docs/reference/index.md`

## Repository Surfaces
- Contribution policy: `CONTRIBUTING.md`
- Security policy: `SECURITY.md`
- Ownership map: `REPO_MAP.md`
- Root contract: `CONTRACT.md`
