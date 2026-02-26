# Canonical Workflows

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

Use these canonical pages for core workflows:

- Running locally: [Local Stack (Make Only)](local-stack.md)
- Deploying to kind: [E2E Stack](e2e/stack.md)
- Running k6: [Load k6](load/k6.md)
- Validating observability: [Observability Acceptance Gates](observability/acceptance-gates.md)
- Updating or rolling back a release pointer: [Release Workflows](release-workflows.md)

All workflow docs should link to these canonical pages instead of deep-linking between random files.

Primary entrypoint target: `ops-full`.
