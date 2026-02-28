# API Troubleshooting

Owner: `api-contracts`  
Type: `guide`  
Surface version: `v1`  
Reason to exist: centralize common API failure diagnosis and resolution.

## Common Failures

- `InvalidCursor`: cursor does not match request contract.
- `MissingDatasetDimension`: dataset identity parameters are incomplete.
- `QueryRejectedByPolicy`: request exceeds policy limits.
- `NotReady`: runtime is alive but not yet ready to serve.

## Example

```bash
curl -i -fsS 'http://127.0.0.1:8080/readyz'
```

## Related References

- [Errors Reference](../reference/errors.md)
- [Schemas Reference](../reference/schemas.md)
