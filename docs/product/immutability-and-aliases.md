# Immutability And Aliases

- Owner: `bijux-atlas-store` + `bijux-atlas-api`

## Dataset Immutability

Published datasets are immutable.

- Catalog publication is append-only for dataset artifacts.
- A published dataset identifier (`release/species/assembly`) must never be overwritten.
- Corrections require publishing a new dataset identity and deprecating the old entry.

## Latest Alias Policy

`latest` is an explicit alias, not an implicit default.

- API requests must provide explicit dataset dimensions unless they target the explicit `latest` alias endpoint.
- Alias resolution must be deterministic from catalog state at request time.
- Alias changes do not mutate historical dataset artifacts.

## Enforcement

- Publish-time no-overwrite checks.
- Catalog validation and deterministic sorting.
- API validation rejects implicit defaults.
