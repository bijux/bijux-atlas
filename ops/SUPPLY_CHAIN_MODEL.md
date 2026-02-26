# Supply Chain Model

- Owner: `bijux-atlas-operations`
- Purpose: `describe trusted inputs, pin sets, and verification points for ops supply-chain governance`
- Consumers: `checks_ops_final_polish_contracts`

## Trusted Inputs

- `ops/inventory/toolchain.json`
- `ops/inventory/pins.yaml`
- `ops/inventory/pin-freeze.json`
- `.github/workflows/*.yml` action pins
- container image references in ops runtime packs and manifests

## Verification Points

- `checks_ops_workflows_github_actions_pinned`
- `checks_ops_image_references_digest_pinned`
- `checks_ops_workflow_routes_dev_atlas`
- `checks_ops_final_polish_contracts`

## Model Boundaries

- Static repo verification covers pin correctness and drift prevention.
- Runtime provenance and hosted-runner execution proof require CI execution evidence outside this repository.
