# Default Field Set

- Owner: `api-contracts`
- Type: `guide`
- Audience: `user`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: explain default and include-driven fields for gene list responses.

## Default behavior

`/v1/genes` returns a minimal field set by default. Additional fields are opt-in via `include`.

## Include usage

```bash
curl -fsS 'http://127.0.0.1:8080/v1/genes?release=110&species=homo_sapiens&assembly=grch38&include=coords,counts&limit=5'
```

Use only include tokens documented in the endpoint contract.

## Canonical contract

- [Reference Contracts Endpoints](../reference/contracts/endpoints.md)
- [Reference Schemas](../reference/schemas.md)

## Next

- [Quick Reference](quick-reference.md)
- [Reference](../reference/index.md)
