# Config Ownership Guide

`configs/` is owned through machine-readable registries and human-readable review rules.

Authoritative ownership sources:
- `configs/registry/owners.json` maps files and groups to owners.
- `configs/registry/owners/identities.json` records owner identities used by those mappings.
- `configs/registry/inventory/configs.json` records the owning group for each config family.

Expectations:
- Every durable config surface needs an owner before new consumers depend on it.
- Ownership changes should land with the registry update in the same commit.
- If a file has no obvious long-term owner, it should not become a stable config surface yet.

Narrative policy belongs here and in `docs/`. The machine registries remain the source of truth for tooling.
