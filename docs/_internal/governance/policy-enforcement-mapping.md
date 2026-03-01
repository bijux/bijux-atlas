# Policy Enforcement Mapping

Owner: `docs-governance`  
Type: `policy`  
Reason to exist: map documented policies to concrete enforcement surfaces.

## Policy To Enforcement

| Policy | Enforcement location |
| --- | --- |
| Root and layout contracts | `bijux dev atlas check root-shape`, `make check` |
| Make workflow governance | workflow checks under `crates/bijux-dev-atlas` |
| Contract drift controls | `make contracts`, `make contracts` |
| OpenAPI drift controls | `make contracts-docs` |
| Docs integrity | `make docs`, `bijux dev atlas docs link-check` |
| Policy schema drift | `bijux dev atlas policies schema-drift` |
| Exception registry checks | `configs/policy/policy-relaxations.json` and policy checks |
| Crate boundary controls | crate guardrail tests and architecture checks |

## Operational Relevance

This mapping ensures every policy statement has a verifiable enforcement path.
