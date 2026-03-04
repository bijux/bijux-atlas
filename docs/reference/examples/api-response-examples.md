# API Response Examples

- Owner: `api-contracts`
- Audience: `user`
- Type: `reference`
- Stability: `stable`
- Reason to exist: provide canonical API response examples for list, filtered, and projected query requests.

## List response example

```json
{
  "data": [
    {
      "gene_id": "ENSG00000139618",
      "symbol": "BRCA2",
      "chromosome": "13",
      "start": 32315474,
      "end": 32400266,
      "biotype": "protein_coding"
    }
  ],
  "page": {
    "limit": 1,
    "next_cursor": "opaque-cursor-token"
  }
}
```

## Projection response example

```json
{
  "data": [
    {
      "gene_id": "ENSG00000139618",
      "symbol": "BRCA2",
      "chromosome": "13"
    }
  ],
  "page": {
    "limit": 1
  }
}
```

## Related

- [API Request Examples](api-request-examples.md)
- [Errors](../errors.md)
