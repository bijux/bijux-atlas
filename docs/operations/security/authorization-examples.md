# Authorization Examples

- Owner: `bijux-atlas-security`
- Type: `example`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: provide concrete role and permission examples.

## Role Assignment Example

```yaml
schema_version: 1
assignments:
  - principal: user
    role_id: role.user.readonly
  - principal: operator
    role_id: role.operator.admin
```

## Permission Definition Example

```yaml
- id: perm.dataset.read
  action: dataset.read
  resource_kind: dataset-id
  description: Read dataset-backed endpoints.
```

## Policy Rule Example

```yaml
- id: AUTHZ-OPS-ADMIN
  effect: allow
  principals: [operator, ci]
  actions: [ops.admin]
  resources:
    kinds: [namespace]
    values: ["*"]
  routes: [/debug, /v1/_debug]
```
