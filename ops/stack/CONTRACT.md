# Contract

- Area: `ops/stack`
- schema_version: `1`

Canonical parent contract: `ops/CONTRACT.md`.

## Invariants

- Canonical image pin source is `ops/inventory/pins.yaml`.
- `ops/stack/version-manifest.json` must mirror pinned image values.
- `ops/inventory/toolchain.json` image map must mirror the same pinned values.
- Stack profiles include `minimal`, `ci`, and `perf`.
- Kind cluster configs include `cluster-small.yaml`, `cluster-dev.yaml`, and `cluster-perf.yaml`.
