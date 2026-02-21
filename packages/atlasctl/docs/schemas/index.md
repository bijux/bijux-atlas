# Schema Catalog and Versioning

Schema catalog source of truth is:

- `src/atlasctl/contracts/schemas/catalog.json`

Schema files live under:

- `src/atlasctl/contracts/schemas/*.schema.json`

## Policy

- Schema names are versioned and immutable (for example `atlasctl.commands.v1`).
- Changes that break compatibility require a version bump.
- `atlasctl validate-output --schema <name> --file <payload.json>` validates by schema name.
- Contract checks validate catalog integrity and sample payloads.
- See [Schema Versioning Policy](versioning-policy.md) for naming and compatibility rules.
