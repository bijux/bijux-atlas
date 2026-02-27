# Configs Contract

## Purpose

`configs/` is the source of truth for repository configuration inputs consumed by `bijux dev atlas configs`.

## Required Surface

- `configs/README.md`
- `configs/INDEX.md`
- `configs/CONTRACT.md`
- `configs/NAMING.md`
- `configs/OWNERS.md`
- `configs/schema/`
- `configs/contracts/`
- `configs/inventory/groups.json`
- `configs/inventory/consumers.json`

## Rules

- Structured config files must be parseable (`json`, `yaml`, `yml`, `toml`).
- Schema-backed configs belong under intent-based directories.
- Config depth for governed config files is capped at four levels under `configs/`.
- Top-level `configs/*` groups must be allowlisted in `configs/inventory/groups.json`.
- Every allowlisted config group must have at least one consumer in `configs/inventory/consumers.json`.
- Control-plane schema registry lives under `ops/schemas/configs/` (data only).
- `configs/` holds config inputs; `ops/schemas/configs/` holds validation schemas.
- Legacy governance references such as `bijux dev atlas` are forbidden in config docs and contracts.
