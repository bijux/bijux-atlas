# Docs Linking Rules

These rules keep MkDocs navigation, breadcrumbs, and link stability intact.

## Allowed internal links

- Use site-relative or docs-relative markdown links that resolve under `docs/`.
- Prefer directory-style URLs and `.md` source links. Do not hardcode `.html` targets.
- Use anchors only when the target heading exists.

## Forbidden link patterns

- Markdown links to runtime artifact paths under `artifacts/`.
- Markdown links to `artifacts/docs/generated/` or `artifacts/ops/`.
- Markdown links to `.md` files outside `docs/`.
- Direct links to `github.com/.../blob/...` for docs pages when a site-relative page exists.
- Absolute links to the same `site_url` domain from docs pages.

## Generated content rules

- Human-facing generated reports must be written under `docs/_internal/generated/` or `docs/_generated/`.
- Generated markdown must contain generator headers and manual-edit prohibition markers.
- Include fragments (`--8<--`) must resolve to existing files under `docs/`.
- Include fragments should avoid `./` links and use stable docs-root-relative paths.

## Autorewrite policy

When `docs links` runs with `--allow-write`, known artifact link patterns are rewritten to governed docs locations.

Current rewrites:
- `artifacts/docs/generated/...` -> `docs/_internal/generated/...`
- `artifacts/tutorials/real-data-overview.md` -> `docs/_internal/generated/real-data-runs-overview.md`
