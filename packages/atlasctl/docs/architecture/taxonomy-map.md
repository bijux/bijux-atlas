# Atlasctl Taxonomy Map (SSOT)

This file is the SSOT for where concepts live.

- `atlasctl.checks.*`: pure check definitions and check execution wiring.
- `atlasctl.checks.layout.policies.*`: layout policy checks.
- `atlasctl.core.*`: runtime primitives/effects/model/schema helpers.
- `atlasctl.contracts.schema.*`: schema catalog, schema files, schema validation.
- `atlasctl.contracts.output.*`: output payload models and output validation.
- `atlasctl.contracts.ids`: canonical schema IDs/versioned contract IDs.
- `atlasctl.policies.budgets.*`: budget policy checks/report handlers.
- `atlasctl.policies.scans.*`: repository scanning helpers.
- `atlasctl.policies.errors.*`: policy-specific error contracts only.
- `atlasctl.policies.command`: CLI glue and dispatch only.

Placement rules:
- New schema IDs must be added only to `atlasctl.contracts.ids`.
- New budget checks must be added under `atlasctl.policies.budgets`.
- New scan helpers must be added under `atlasctl.policies.scans`.
