# Registry Is The SSOT Spine

`atlasctl` registry data is authored once and loaded through a single typed loader.

## Author Inputs

- Checks: `packages/atlasctl/src/atlasctl/checks/REGISTRY.toml`
- Owners: `configs/meta/owners.json`
- Suites: `packages/atlasctl/src/atlasctl/registry/suites_catalog.json` (first-class suite catalog)
- Budgets: `pyproject.toml` (`tool.atlasctl.budgets`)
- Commands/capabilities: command surface registry + runtime capability derivation

## One Loader

- Use `atlasctl.registry.loader.load()` (typed `Registry`)
- Models live in `atlasctl.registry.models`
- Selector helpers live in `atlasctl.registry.selectors`

## Generated Outputs (Do Not Hand Edit)

- `packages/atlasctl/src/atlasctl/registry/registry_spine.generated.json`
- `packages/atlasctl/src/atlasctl/checks/REGISTRY.generated.json`
- `packages/atlasctl/src/atlasctl/registry/checks_catalog.json`
- `docs/_generated/cli.md`
- `packages/atlasctl/docs/checks/index.md`
- `packages/atlasctl/docs/ownership.md`

Generate and verify:

- `./bin/atlasctl registry gen`
- `./bin/atlasctl registry diff`
- `./bin/atlasctl registry validate`
- `./bin/atlasctl registry gate`

## Adding Items

1. Add/modify author input (check entry, owner, suite, command metadata source).
2. Run `./bin/atlasctl registry gen`.
3. Run `./bin/atlasctl registry gate`.
4. Commit generated outputs with the author-input change.

## Safe Check-ID Migration

- Dry-run rewrite: `./bin/atlasctl registry rename-check-id --json`
- Apply rewrite: `./bin/atlasctl registry rename-check-id --apply`

## Catalog Duplication Policy

`checks_catalog.json` and `suites_catalog.json` are currently retained because they back existing consumers and are not redundant yet. The spine composes them through one loader and can replace them in a later cutover.
