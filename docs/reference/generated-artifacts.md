# Generated Artifacts

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `reference`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: list the committed generated documentation artifacts and their generators.

| Artifact | Source |
| --- | --- |
| `docs/reference/repo-map.md` | `scripts/docs/generate_repo_map.py` |
| `docs/_internal/generated/*` | control-plane and docs generation commands |
| `ops/_examples/networkpolicy/*.yaml` | Helm render snapshots validated by tests |

## `_generated` Policy

Artifacts under `docs/_generated` are not the canonical docs source.

- Markdown aliases under `docs/_generated/` may redirect through MkDocs when they point to a
  markdown page in `docs/_internal/generated/`.
- JSON and other machine-readable artifacts under `docs/_generated/` are build outputs or
  committed diagnostics. They stay generator-owned and are not emitted through `mkdocs-redirects`.
