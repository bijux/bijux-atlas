# Repo Hygiene

`repo-hygiene` is the baseline repository integrity contract for deterministic local and CI runs.

## What is enforced

- no `__pycache__/` directories under `packages/`
- no `*.pyc` files anywhere in the repository
- no `.pytest_cache/` directories anywhere in the repository
- no tracked files under `ops/_evidence/`
- no tracked files under `ops/_artifacts/` except approved examples
- no tracked files under `ops/_generated/`
- tracked `configs/_generated/` content requires `configs/_generated/checksums.json`
- no duplicate tracked paths that differ only by filename case
- no tracked symlinks outside `configs/repo/symlink-allowlist.json`
- no tracked files larger than 5 MB
- no generated timestamp fields in tracked generated JSON/YAML outputs
- no command code paths that target `ops/_evidence/` or `ops/_artifacts/` writes

## Commands

```bash
./bin/atlasctl check repo-hygiene
./bin/atlasctl doctor repo-hygiene
./bin/atlasctl fix hygiene --apply
```

## CI lane

- `repo-hygiene-fast` runs:
  - `./bin/atlasctl fix hygiene --apply`
  - `./bin/atlasctl doctor repo-hygiene`
  - `./bin/atlasctl suite run repo-hygiene`
  - `git diff --exit-code`
