# Docs build model

Documentation follows a build, preview, and publish flow.

## Build

```bash
make docs-generate
make docs-check
```

Build validates metadata, links, and section contracts before preview.

## Preview

```bash
make docs-serve
```

Preview uses the curated nav and excludes `_generated`, `_drafts`, `_meta`, and `_nav` from reader surfaces.

## Publish

- Merge to default branch only after docs checks pass.
- Publish only curated reader sections.
- Keep governance and generated artifacts contributor-only.
