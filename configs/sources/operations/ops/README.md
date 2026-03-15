# Operations configs

- Owner: `bijux-atlas-operations`
- Purpose: define runtime, deployment, observability, SLO, and operational policy inputs.
- Consumers: operations validation, Helm/Kubernetes checks, and runtime support tooling.
- Update workflow: update the specific ops config with matching operational evidence, then rerun ops validation and relevant contracts.
