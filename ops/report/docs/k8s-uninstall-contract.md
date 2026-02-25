# Kubernetes Uninstall Contract

- Owner: bijux-atlas-operations
- Stability: stable

## Command

- `bijux dev atlas ops stack down --profile <name> --allow-subprocess --allow-write`

## Rules

- Uninstall follows the same profile identity used for install.
- Uninstall must leave no managed release resources in target namespace.
- Uninstall evidence is recorded under artifacts for the run id.
