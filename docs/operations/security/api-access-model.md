# API Access Model

- Owner: `bijux-atlas-security`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define endpoint audiences and the default exposure expectations.

## Publicly reachable health surface

These endpoints stay reachable for platform health checks even when runtime auth is enabled:

- `/healthz`
- `/readyz`
- `/v1/version`

They are intended for platform automation, probes, and load balancers.

## Standard API surface

These endpoints are intended for approved callers behind the deployment boundary:

- `/v1/datasets`
- `/v1/releases/...`
- `/v1/genes`
- `/v1/query/validate`
- `/v1/sequence/region`
- `/v1/diff/...`

These are subject to the configured auth mode and request policy evaluation.

When `auth.mode=oidc` or `auth.mode=mtls`, these routes must be reached through the approved auth
boundary so the trusted identity headers are present.

## Admin endpoint inventory

Admin-style endpoints are:

- `/debug/datasets`
- `/debug/dataset-health`
- `/debug/registry-health`
- `/v1/_debug/echo`

They are disabled by default and require an explicit runtime enable flag.
