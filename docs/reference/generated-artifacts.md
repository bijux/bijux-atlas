# Generated Artifacts

- Owner: `docs-governance`
- Type: `reference`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@fc2319a8483a4d0c9d08e5227ec31d7cb6677c4a`
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
