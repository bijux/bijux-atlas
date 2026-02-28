# Default Field Set

Owner: `api-contracts`  
Type: `guide`  
Surface version: `v1`  
Reason to exist: define the canonical default and optional response fields for `/v1/genes`.

## Contract

- Default fields: `gene_id`, `name`.
- Optional include tokens:
  - `coords`
  - `biotype`
  - `counts`
  - `length`

## Example

```bash
curl -fsS 'http://127.0.0.1:8080/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&include=coords,counts&limit=5'
```

## Related References

- [Schemas Reference](../reference/schemas.md)
- [Errors Reference](../reference/errors.md)
