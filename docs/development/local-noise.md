# Local Noise Policy

- Owner: `docs-governance`

## What

Defines local-only files/directories that are tolerated in developer workspaces.

## Local Noise Entries

- `.idea/`: editor metadata, ignored locally, never tracked.
- `.DS_Store`: OS metadata, ignored locally, never tracked.
- `target/`: local build cache may exist, but must never be tracked and must not appear in CI workspaces.

## Enforcement

- `.gitignore` includes all local-noise entries.
- `scripts/layout/check_repo_hygiene.sh` fails if these are tracked.
- In CI, root `target/` presence fails hygiene checks.

## Notes

- Local noise is allowed for developer convenience.
- CI policy is strict cleanliness and reproducibility.
