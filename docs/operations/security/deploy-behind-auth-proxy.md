# Deploy Behind Auth Proxy

- Owner: `bijux-atlas-security`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@ecce23744e45d1869232fef0bd682d7b33f80991`
- Reason to exist: define the required ingress boundary when Atlas is deployed beyond a strictly private network.

## Prereqs

- An ingress controller or reverse proxy in front of Atlas.
- A policy decision on whether Atlas built-in auth is `disabled`, `api-key`, or `hmac`.

## Install

Terminate user-facing or cross-boundary authentication at the ingress layer first.

Concrete accepted patterns:

- NGINX external auth or forward-auth
- Traefik forward-auth middleware
- Service-mesh authn/authz policy

Pass only the identity headers that the deployment has explicitly approved.

## Verify

- Confirm Atlas is not directly exposed on a public listener without the proxy.
- Confirm the ingress layer rejects unauthenticated callers before forwarding.
- If Atlas built-in auth is enabled, confirm the proxy and Atlas mode agree on the expected caller contract.

## Rollback

- Remove the new proxy policy only after restoring the previous protected ingress policy.
- Do not leave Atlas externally reachable without an equivalent boundary control.
