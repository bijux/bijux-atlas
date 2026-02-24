# Configs Contract

## Purpose

`configs/` is the source of truth for repository configuration inputs consumed by `bijux dev atlas configs`.

## Required Surface

- `configs/README.md`
- `configs/INDEX.md`
- `configs/CONTRACT.md`
- `configs/schema/`
- `configs/contracts/`

## Rules

- Structured config files must be parseable (`json`, `yaml`, `yml`, `toml`).
- Schema-backed configs belong under intent-based directories.
- Control-plane schema registry lives under `ops/schemas/configs/` (data only).
- `configs/` holds config inputs; `ops/schemas/configs/` holds validation schemas.
- Legacy governance references such as `atlasctl` are forbidden in config docs and contracts.
