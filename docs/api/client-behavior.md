# Client Behavior Expectations

Owner: `api-contracts`  
Type: `guide`  
Surface version: `v1`  
Reason to exist: define retry, caching, and idempotent usage expectations for API clients.

## Expectations

- Send explicit dataset identity for dataset-backed queries.
- Reuse cursor tokens only with the matching query contract.
- Honor server cache headers and ETag responses.
- Retry transient failures conservatively; avoid retry storms.

## Example

```bash
curl -fsS 'http://127.0.0.1:8080/v1/query/validate' \
  -H 'content-type: application/json' \
  -d '{"release":"110","species":"homo_sapiens","assembly":"GRCh38","limit":10}'
```

## Related References

- [Schemas Reference](../reference/schemas.md)
- [Errors Reference](../reference/errors.md)
