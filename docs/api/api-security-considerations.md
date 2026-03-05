# API Security Considerations

- Owner: `api-contracts`
- Type: `policy`
- Audience: `user`
- Stability: `stable`

## Guidance

- Validate inputs against documented constraints.
- Avoid exposing internal debug endpoints in public environments.
- Treat deprecated endpoints as migration-only, not long-term dependencies.
- Keep clients pinned to verified API contract versions.
