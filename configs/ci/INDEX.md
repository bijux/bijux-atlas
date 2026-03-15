# CI Config Index

This directory defines CI lane routing, workflow metadata, and environment contracts consumed by repository automation.

Key files:
- `env-contract.json` declares the CI environment contract checked by runtime and release tooling.
- `lane-surface.json` maps lanes to workflows and reporting surfaces.
- `lanes.json` defines the lane registry used by CI governance checks.
- `policy-exceptions.json` and `policy-outside-control-plane.json` record deliberate CI policy deviations.
- `workflow-allowlist.json` and `workflow-step-patterns.json` constrain supported workflow structure.

Change these files together with the workflow change they govern so CI policy and actual workflow behavior stay aligned.
