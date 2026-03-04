# Generated Artifacts

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `reference`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: list the committed generated documentation artifacts and their generators.

| Artifact | Source |
| --- | --- |
| `docs/reference/repo-map.md` | `bijux-dev-atlas docs reference generate --allow-subprocess --allow-write` |
| `docs/_internal/generated/*` | control-plane and docs generation commands |
| `ops/k8s/examples/networkpolicy/*.yaml` | Helm render snapshots validated by tests |

## `_generated` Policy

Artifacts under `docs/_generated` are not the canonical docs source.

- Markdown aliases under `docs/_generated/` may redirect through MkDocs when they point to a
  markdown page in `docs/_internal/generated/`.
- JSON and other machine-readable artifacts under `docs/_generated/` are build outputs or
  committed diagnostics. They stay generator-owned and are not emitted through `mkdocs-redirects`.
