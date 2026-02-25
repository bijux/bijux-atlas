# Contract

- Area: `ops/stack`
- schema_version: `1`

Canonical parent contract: `ops/CONTRACT.md`.

## Authored vs Generated

| Path | Role |
| --- | --- |
| `ops/inventory/pins.yaml` | Authored SSOT for image and version pins |
| `ops/stack/profiles.json` | Authored stack profile definitions |
| `ops/stack/stack.toml` | Authored stack composition |
| `ops/stack/generated/version-manifest.json` | Generated stack image manifest derived from inventory pins |
| `ops/stack/generated/stack-index.json` | Generated stack artifact index |
| `ops/stack/generated/dependency-graph.json` | Generated dependency graph |
| `ops/stack/generated/artifact-metadata.json` | Generated artifact metadata |
| `ops/stack/generated/versions.json` | Generated stack version summary |

## Schema References

| Artifact | Schema |
| --- | --- |
| `ops/stack/profiles.json` | `ops/schema/stack/profile-manifest.schema.json` |
| `ops/stack/generated/version-manifest.json` | `ops/schema/stack/version-manifest.schema.json` |
| `ops/stack/generated/dependency-graph.json` | `ops/schema/stack/dependency-graph.schema.json` |
| `ops/stack/generated/artifact-metadata.json` | `ops/schema/stack/artifact-metadata.schema.json` |
| `ops/inventory/pins.yaml` | `ops/schema/inventory/pins.schema.json` |

## Invariants

- Canonical image pin source is `ops/inventory/pins.yaml`.
- Pins outside `ops/inventory/pins.yaml` are forbidden.
- `ops/stack/generated/version-manifest.json` must mirror pinned image values.
- `ops/inventory/toolchain.json` image map must mirror the same pinned values.
- Stack profiles include `minimal`, `ci`, and `perf`.
- Kind cluster configs include `cluster-small.yaml`, `cluster-dev.yaml`, and `cluster-perf.yaml`.
- Pin keys must be canonical and stable across releases.
- Pin values must use stable formats (image digests or pinned versions).
- Pin lifecycle is governed by `ops/inventory/pin-freeze.json`.
