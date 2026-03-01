# Where truth lives

Use one source of truth per docs governance artifact.

## Authoritative files

- Section ownership: `docs/_internal/registry/owners.json`
- Documentation registry: `docs/_internal/registry/registry.json`
- Top-level section map: `docs/_internal/registry/sections.json`
- Front matter inventory: `docs/_internal/governance/metadata/front-matter.index.json` (generated from `docs/_internal/registry/registry.json`)

## Rules

- Do not create duplicate `owners.json`, `registry.json`, or `sections.json` files under `docs/`.
- Generated artifacts in `docs/_internal/generated/` are diagnostics, not canonical governance state.
- Redirect any moved governance metadata paths through `docs/redirects.json`.

## Verification

```bash
cargo test -q -p bijux-dev-atlas --test docs_registry_contracts -- --nocapture
```

## Next steps

- [Docs change process](../_internal/meta/docs-change-process.md)
- [Docs governance](../_internal/governance/index.md)
