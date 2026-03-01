# Repository Constitution

- Owner: `bijux-atlas-operations`
- Audience: `maintainers`
- Stability: `stable`

## Purpose

This document is the canonical human-readable constitution for repository-wide rules.
Executable authority remains in contracts and check registries.

## Canonical Root Lobby

Root is a lobby, not a document archive.
Only canonical entry files and directories are permitted at root.
Generated references must live under `docs/reference/`.

## Rule Location

- Repo laws source: `docs/_internal/contracts/repo-laws.md`
- Root surface manifest: `ops/inventory/root-surface.json`
- Root markdown allowlist: `configs/repo/root-file-allowlist.json`

## Change Protocol

Any root surface expansion requires:

1. allowlist and manifest updates
2. contract check pass
3. constitution rationale update in `docs/_internal/root-surface.md`

