# Check Domains

Domain names are policy ownership and intent boundaries.

Contract taxonomy (architecture-level intent set):

- `root`
- `python`
- `docs`
- `make`
- `ops`
- `policies`
- `product`

Runtime registry taxonomy (current executable set) currently includes:

- `repo`: repository shape, architecture boundaries, dependency and layout policy
- `docs`: docs integrity, references, and generated documentation drift
- `ops`: ops/runtime contracts, manifests, and operational policy
- `make`: make wrapper contracts and command delegation policy
- `configs`: config schema and compiler input policy
- `contracts`: schema and contract catalog integrity
- `docker`: docker policy and layout checks
- `checks`: self-validation for check registry and check ecosystem
- `policies`: policy data and bypass governance checks
- `license`: license policy checks
- `python`: python-specific runtime or toolchain checks

Canonical check id format encodes domain:

- `checks_<domain>_<area>_<intent>`

Supporting terms:

- Suite IDs are registry entries, not directory names.
- Visibility values are `PUBLIC` and `INTERNAL`.
- Speed values are `FAST` and `SLOW` with `FAST` as default.
