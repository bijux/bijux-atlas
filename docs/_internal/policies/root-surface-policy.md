# Root Surface Policy

- Owner: `bijux-atlas-platform`
- Audience: `contributors`

## Allowed Root Markdown

Canonical source: `configs/layout/root-markdown-allowlist.json`

## Change Rule

No new root markdown files may be added unless all are updated in one atomic change:
- `configs/layout/root-markdown-allowlist.json`
- `configs/repo/root-file-allowlist.json`
- `docs/_internal/ssot.md` (if boundary semantics changed)
