# SDK strategy

- Owner: `api-contracts`
- Type: `policy`
- Audience: `user`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: remove ambiguity about how clients should integrate with Atlas.

## Position

Atlas does not publish an official SDK at this time. The compatibility boundary is the documented HTTP API plus the published OpenAPI surface.

## What to build against

- HTTP endpoints documented in [V1 surface](v1-surface.md)
- Error handling documented in [Errors](errors.md)
- Compatibility guarantees documented in [Compatibility](compatibility.md)

## Consumer guidance

- Generate client bindings from the published OpenAPI surface if you need typed clients.
- Keep generated bindings scoped to documented endpoints and fields.
- Treat undocumented fields and debug endpoints as unsupported.

## Next steps

- [Quick reference](quick-reference.md)
- [Compatibility](compatibility.md)
