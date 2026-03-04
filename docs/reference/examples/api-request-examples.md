# API Request Examples

- Owner: `api-contracts`
- Audience: `user`
- Type: `reference`
- Stability: `stable`
- Reason to exist: provide canonical API request examples for query, pagination, filtering, and projection behavior.

## List genes request

```http
GET /v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=2 HTTP/1.1
Host: 127.0.0.1:8080
Accept: application/json
```

## Filtered request

```http
GET /v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&chromosome=13&biotype=protein_coding HTTP/1.1
Host: 127.0.0.1:8080
Accept: application/json
```

## Projection request

```http
GET /v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&fields=gene_id,symbol,chromosome HTTP/1.1
Host: 127.0.0.1:8080
Accept: application/json
```

## Related

- [API Response Examples](api-response-examples.md)
- [Query Parameters](../querying/query-parameters.md)
