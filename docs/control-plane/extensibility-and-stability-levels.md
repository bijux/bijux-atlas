# Extensibility and stability levels

- Owner: `platform`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define which control-plane surfaces are safe to extend freely and which require compatibility care.

## Stability levels

- stable: command names, report fields, and documented make wrappers relied on by CI or contributors
- evolving: documented contributor surfaces that can grow compatibly but should avoid unnecessary churn
- internal: implementation details and diagnostics not intended as public extension points

## Extension guidance

- extend stable surfaces additively
- avoid renaming stable commands or report keys without redirects or compatibility migration
- keep internal audit dumps behind dashboard-only entrypoints

## Verify success

Use this page during review when deciding whether a change is additive or breaking.

## Next steps

- [Contract changes and versioning](contract-changes-and-versioning.md)
- [Reports contract](reports-contract.md)
- [Known limitations](known-limitations.md)
