# Dataset Access Policies

- Owner: `bijux-atlas-security`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: define authorization for dataset read and query paths.

## Allowed Data-Plane Actions

- `catalog.read` for discovery routes
- `dataset.read` for dataset-backed endpoints

## Endpoint Scope

Dataset access policy applies to:

- `/v1/datasets`
- `/v1/releases`
- `/v1/genes`
- `/v1/sequence`
- `/v1/diff`
- `/v1/transcripts`

## Principal Rules

- `user` and `service-account` are limited to read actions.
- `operator` inherits read access and may hold administrative permissions.
