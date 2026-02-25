# ops/inventory

Canonical inventory documents for ops ownership, command surface, namespaces, toolchain, release pins, image pins, drill catalog, contracts, and layer policy.

- `ops/inventory/contracts-map.json` is the authoritative inventory contract map.
- `ops/inventory/contracts.json` is the generated mirror from the contract map.

- Image pins and dataset pins are owned only by `ops/inventory/pins.yaml`.
- Tool probing and tool image inventory are owned by `ops/inventory/toolchain.json`.
- GC pin input is owned by `ops/inventory/gc-pins.json`.
- Surface, owners, and drills inventories are canonical in:
  - `ops/inventory/surfaces.json`
  - `ops/inventory/owners.json`
  - `ops/inventory/drills.json`
- Gate definitions are canonical in:
  - `ops/inventory/gates.json`
- Release pin freeze state is canonical in:
  - `ops/inventory/pin-freeze.json`
