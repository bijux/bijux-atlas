# Inventory configs

- Owner: `platform`
- Purpose: define the authoritative inventory of config groups and the file patterns each group governs.
- Consumers: configs contracts, registry indexing, and generated configs index output.
- Update workflow: update the inventory entry with the file move or new config, update owners/consumers/schema maps if the path contract changed, then rerun configs contracts and refresh generated indexes.

Registry split:
- `configs.json` is the group and file-pattern inventory.
- `owners.json` is the inventory-owned group owner map used by contracts.
- `consumers.json` is the inventory-owned consumer map used by contracts.
- `index.json` points tooling at the canonical registry files.
- `no-schema-justifications.json` records approved exceptions for files without schema coverage.

## Ownership registries

- `docs-owners.json`: canonical docs section owners.
- `ops-owners.json`: canonical ops section owners.
