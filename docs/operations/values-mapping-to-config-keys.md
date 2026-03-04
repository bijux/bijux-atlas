# Values mapping to config keys

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: map chart-facing values to the canonical runtime config keys without duplicating the config reference.

## Mapping rules

- chart values select deployment-time wiring
- runtime config keys define process behavior
- reference config keys remain canonical for field definitions

## Key mappings

| Chart concern | Runtime config reference |
| --- | --- |
| image and release identity | [Reference configs](../reference/configs.md) |
| telemetry endpoints | [Reference configs](../reference/configs.md) |
| storage and persistence | [Reference configs](../reference/configs.md) |
| request and resource limits | [Reference configs](../reference/configs.md) |

## Verify success

```bash
make ops-values-validate
```

## Next steps

- [Minimal production overrides](minimal-production-overrides.md)
- [Deploy](deploy.md)
- [Reference configs](../reference/configs.md)
