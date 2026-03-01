# Human vs Machine Contracts

- Machine mode: JSON responses with stable top-level shape and stable error codes.
- Human mode: pretty formatting is opt-in via `pretty=true`; default is compact JSON.
- No surprise fields: responses avoid implicit extra fields in v1 contracts.
- Content negotiation: only `application/json` is supported for v1.
