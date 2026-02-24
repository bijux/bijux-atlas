# Docs Contract

## Purpose

Define the canonical documentation surface and validation rules for repository docs.

## SSOT Inputs

- `mkdocs.yml` defines docs navigation and published page surface
- `docs/` contains markdown content and generated docs outputs committed to git
- `docs/contracts/plugin/mode.md` defines runtime/dev command dispatch contract pages
- `crates/bijux-atlas-cli/docs/CLI_COMMAND_LIST.md` and `crates/bijux-dev-atlas/docs/CLI_COMMAND_LIST.md` define command surface lists

## Required Files

- `docs/INDEX.md`
- `docs/contracts/INDEX.md`
- `docs/contracts/plugin/mode.md`
- `mkdocs.yml`

## Rules

- `mkdocs.yml` must parse and all nav paths must reference real files under `docs/`
- Docs page names must be intent-based and deterministic (no spaces; lowercase except `README.md` / `INDEX.md`)
- Docs must not introduce legacy governance references (`atlasctl`, `scripts/areas`, `xtask`, legacy atlasctl make targets)
- Runtime and dev command surface docs must exist and match their command contracts/help snapshots
- Orphan markdown pages must be avoided unless intentionally excluded from published docs surface
