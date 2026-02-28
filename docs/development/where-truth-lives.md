# Where truth lives

Use one source of truth per docs governance artifact.

## Authoritative files

- Section ownership: `docs/owners.json`
- Documentation registry: `docs/registry.json`
- Top-level section map: `docs/sections.json`
- Front matter inventory: `docs/governance/metadata/front-matter.index.json` (generated from `docs/registry.json`)

## Rules

- Do not create duplicate `owners.json`, `registry.json`, or `sections.json` files under `docs/`.
- Generated artifacts in `docs/_generated/` are diagnostics, not canonical governance state.
- Redirect any moved governance metadata paths through `docs/redirects.json`.

## Verification

```bash
cargo test -q -p bijux-dev-atlas --test docs_registry_contracts -- --nocapture
```

## Next steps

- [Docs change process](../_meta/docs-change-process.md)
- [Docs governance](../governance/index.md)
