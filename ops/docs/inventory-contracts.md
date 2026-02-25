# Ops Inventory Contract Map

Canonical inventory SSOT files and their schemas/generators.

| Inventory file | Schema | Generator / owner | Notes |
|---|---|---|---|
| `ops/inventory/owners.json` | `ops/schema/meta/ownership.schema.json` | manual + owner docs sync | Replaces `ops/inventory/owners.json` |
| `ops/inventory/surfaces.json` | `configs/ops/public-surface.json` (surface semantics) | `bijux dev atlas ops gen run` / surface generator | Canonical ops surface inventory |
| `ops/inventory/contracts.json` | `ops/schema/meta/artifact-allowlist.schema.json` (structure checked separately) | manual | References top-level + domain contract artifacts |
| `ops/inventory/layers.json` | `ops/schema/meta/layer-contract.schema.json` | `bijux dev atlas ops gen run` / layer contract generator | Canonical layer contract inventory |
| `ops/inventory/pins.yaml` | `ops/schema/inventory/pins.schema.json` | `bijux dev atlas ops generate pins-index` | Canonical image and dataset pin SSOT |
| `ops/inventory/toolchain.json` | `ops/schema/inventory/toolchain.schema.json` | manual (validated in `ops doctor`) | Canonical toolchain probe and image inventory |
| `ops/inventory/toolchain.yaml` | `ops/schema/stack/version-manifest.schema.json` + tool-versions SSOT | `bijux dev atlas ops stack versions-sync` + migration tooling | Compatibility mirror during migration window |
| `ops/inventory/owner-docs.fragments.json` | n/a (generated fragment inventory) | `bijux dev atlas ops migrate` | Derived from `ops/*/OWNER.md` during migration window |
| `ops/inventory/contracts/*.contract.fragment.json` | n/a (fragment contract format) | `bijux dev atlas ops migrate` | Structured fragments replacing per-domain free-form contract docs over time |
