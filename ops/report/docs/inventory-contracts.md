# Ops Inventory Contract Map

Canonical inventory SSOT files and their schemas/generators.

| Inventory file | Schema | Generator / owner | Notes |
|---|---|---|---|
| `ops/inventory/owners.json` | `ops/schema/inventory/owners.schema.json` | manual + owner docs sync | Canonical ownership map |
| `ops/inventory/surfaces.json` | `ops/schema/inventory/surfaces.schema.json` | `bijux dev atlas ops gen run` / surface generator | Canonical ops surface inventory |
| `ops/inventory/contracts.json` | `ops/schema/inventory/contracts.schema.json` | generated from `contracts-map.json` | Generated contract registry mirror |
| `ops/inventory/layers.json` | `ops/schema/inventory/layers.schema.json` | `bijux dev atlas ops gen run` / layer contract generator | Canonical layer contract inventory |
| `ops/inventory/pins.yaml` | `ops/schema/inventory/pins.schema.json` | `bijux dev atlas ops generate pins-index` | Canonical image and dataset pin SSOT |
| `ops/inventory/toolchain.json` | `ops/schema/inventory/toolchain.schema.json` | manual (validated in `ops doctor`) | Canonical toolchain probe and image inventory |
| `ops/inventory/tools.toml` | `ops/schema/inventory/tools.schema.json` | manual | External tool probe registry only |
| `ops/inventory/registry.toml` | `ops/schema/inventory/registry.schema.json` | manual | Check catalog registry; not a tool probe registry |
| `ops/inventory/owner-docs.fragments.json` | `ops/schema/inventory/owner-docs.fragments.schema.json` | `bijux dev atlas ops migrate` | Derived owner-doc fragment inventory |
| `ops/inventory/contracts/*.contract.fragment.json` | n/a (fragment contract format) | `bijux dev atlas ops migrate` | Structured fragments replacing per-domain free-form contract docs over time |
