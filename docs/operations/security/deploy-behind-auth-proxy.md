# Deploy Behind Auth Proxy

- Owner: `bijux-atlas-security`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define the required ingress boundary when Atlas is deployed beyond a strictly private network.

## Prereqs

- An ingress controller or reverse proxy in front of Atlas.
- A policy decision on whether Atlas auth mode is `disabled`, `api-key`, `oidc`, or `mtls`.

## Install

Terminate user-facing or cross-boundary authentication at the ingress layer first.

Concrete accepted patterns:

- NGINX external auth or forward-auth
- Traefik forward-auth middleware
- Service-mesh authn/authz policy

Pass only the identity headers that the deployment has explicitly approved.

### NGINX ingress example

```yaml
metadata:
  annotations:
    nginx.ingress.kubernetes.io/auth-url: "https://auth.example.internal/check"
    nginx.ingress.kubernetes.io/auth-signin: "https://auth.example.internal/start"
    nginx.ingress.kubernetes.io/auth-response-headers: "X-Forwarded-User,X-Forwarded-Email"
```

### Traefik example

```yaml
metadata:
  annotations:
    traefik.ingress.kubernetes.io/router.middlewares: "atlas-forward-auth@kubernetescrd"
```

## Verify

- Confirm Atlas is not directly exposed on a public listener without the proxy.
- Confirm the ingress layer rejects unauthenticated callers before forwarding.
- If Atlas auth mode is `oidc` or `mtls`, confirm the proxy forwards only the approved identity headers after successful authentication.
- If Atlas auth mode is `api-key`, confirm the proxy preserves the expected application auth header contract without injecting credentials.

## Rollback

- Remove the new proxy policy only after restoring the previous protected ingress policy.
- Do not leave Atlas externally reachable without an equivalent boundary control.
