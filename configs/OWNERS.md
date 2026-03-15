# Config Ownership Guide

`configs/` is owned through machine-readable registries and human-readable review rules.

Authoritative ownership sources:
- `configs/registry/owners.json` maps files and groups to owners.
- `configs/registry/owners/identities.json` records owner identities used by those mappings.
- `configs/registry/inventory/configs.json` records which file patterns belong to each config group.

Expectations:
- Every durable config surface needs an owner before new consumers depend on it.
- Ownership changes should land with the matching inventory, consumer, or schema update in the same commit.
- If a file has no obvious long-term owner, it should not become a stable config surface yet.

Review order:
- check `configs/registry/inventory/configs.json` to see which group governs a path
- check `configs/registry/owners.json` to see who owns the path or group
- check `configs/registry/owners/identities.json` to resolve the owner label to human reviewers

Narrative policy belongs here and in `docs/`. The registries remain the source of truth for tooling.
