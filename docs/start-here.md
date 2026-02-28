# Start Here

Owner: `docs-governance`  
Type: `guide`  
Audience: `user`  
Reason to exist: provide the single onboarding path for Atlas documentation.

This is the only onboarding root in `docs/`.

Use [Docs Home](index.md) for section navigation.

## Quickstart

```bash
bijux dev atlas demo quickstart --format json
```

## FAQ

### Why use `bijux dev atlas` instead of scripts?

To enforce deterministic behavior, typed contracts, and testable governance surfaces.

### Where are generated outputs stored?

Under `artifacts/` with run-id scoped directories.

### Where is the docs source of truth?

Canonical pages under `docs/` with policy controls in `docs/governance/`.

### How do I find stable command surfaces?

Use `make help` and `docs/reference/commands.md`.

## Next Step

- Operators: go to [Run Locally](operations/run-locally.md).
- Contributors: go to [Development](development/index.md).
- API users: go to [API](api/index.md).
