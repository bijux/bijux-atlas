# Make Public Surface

- Owner: `bijux-atlas-platform`
- Tier: `stable`
- Audience: `developers`
- Source-of-truth: `make/help.md`, `make/target-list.json`, `makefiles/root.mk`

This page documents the curated public make targets. Make remains a delegation layer; behavior is enforced by `bijux dev atlas` commands and cargo lanes.

## Canonical Commands

- List targets: `make help`
- Validate make governance: `make doctor`
- Refresh target inventory: `make make-target-list`

## Docker Targets

- `make docker` (alias of `make docker-contracts`)
- `make docker-contracts`
- `make docker-contracts-effect`
- `make docker-gate`

## Policy

- Do not call private/internal make targets from CI.
- Do not add docker commands outside `makefiles/docker.mk`.
- Do not duplicate this list in other docs; point to `make help`.
