# TLS Configuration Examples

- Owner: `bijux-atlas-security`
- Type: `example`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: provide practical TLS examples.

## Runtime Environment

```bash
ATLAS_REQUIRE_HTTPS=true
ATLAS_AUTH_MODE=token
```

## Security Runtime Contract

```yaml
transport:
  tls_required: true
  min_tls_version: "1.2"
```
