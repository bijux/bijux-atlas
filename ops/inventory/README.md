# ops/inventory

Canonical inventory documents for ops ownership, command surface, namespaces, toolchain, release pins, image pins, drill catalog, contracts, and layer policy.

- Image pins and dataset pins are owned only by `ops/inventory/pins.yaml`.
- Tool probing and tool image inventory are owned by `ops/inventory/toolchain.json`.
- Surface, owners, and drills inventories are canonical in:
  - `ops/inventory/surfaces.json`
  - `ops/inventory/owners.json`
  - `ops/inventory/drills.json`
