# Product configs

- Owner: `product`
- Purpose: hold product-level contract manifests and external tool declarations.
- Consumers: product compatibility checks and artifact manifest validation.
- Update workflow: update manifests with the corresponding product contract change, then rerun affected contract suites.
- Boundary: authored product inputs stay at this directory root, and local schemas live under `schemas/`.
