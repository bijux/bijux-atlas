# Runtime Config Sources

This directory groups authored inputs that shape runtime-facing behavior and product surfaces.

Current domains:
- `cli/` for CLI-facing config inputs and examples.
- `clients/` for generated client metadata and client-facing contracts.
- `datasets/` for dataset manifests and ingest policy inputs.
- `openapi/` for versioned OpenAPI snapshots and generated output.
- `product/` for product-manifest inputs consumed by release and artifact tooling.

These paths are authoritative inputs. Generated artifacts and schema contracts live elsewhere.
