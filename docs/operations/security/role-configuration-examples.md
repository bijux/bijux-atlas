# Role Configuration Examples

- Owner: `bijux-atlas-security`
- Type: `example`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: provide reusable role configuration patterns.

## Readonly Role

```yaml
- id: role.user.readonly
  description: End-user read role
  permissions: [perm.catalog.read, perm.dataset.read]
  inherits: []
```

## Operator Role With Inheritance

```yaml
- id: role.operator.admin
  description: Operator admin role
  permissions: [perm.ops.admin, perm.dataset.ingest]
  inherits: [role.user.readonly]
```

## Automation Role

```yaml
- id: role.automation.release
  description: CI release role
  permissions: [perm.ops.admin]
  inherits: [role.service.readonly]
```
