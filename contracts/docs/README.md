# Docs contracts

Docs contracts are enforced by `bijux dev atlas contracts docs --mode static`.

Scope:
- docs tree shape, naming, and entrypoints
- frontmatter and ownership coverage
- link integrity and section reachability
- generated docs artifacts and reproducibility inputs

Related suites:
- `docs_required`: PR-required static docs checks
- `docs`: full docs-domain suite
- `docs_fast`: fast docs checks

Related sources:
- `crates/bijux-dev-atlas/src/contracts/docs/`
- `crates/bijux-dev-atlas/src/commands/docs/`
- `ops/inventory/registry.toml`
