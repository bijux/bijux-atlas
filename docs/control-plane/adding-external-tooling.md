# Adding external tooling

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: define the approval path for introducing a new external binary into control-plane workflows.

## Approval process

1. State the user-facing problem the tool solves.
2. Explain why an existing wrapped tool is insufficient.
3. Define the exact commands and capability flags the tool will require.
4. Add deterministic failure messaging for missing-tool cases.
5. Wire the tool behind a stable wrapper or command family.

## Review rules

- do not add a binary just to hide shell complexity
- prefer one canonical wrapper target over multiple ad-hoc script call sites
- document lane scope so expensive tools do not leak into cheap local loops

## Verify success

```bash
make ops-tools-verify
```

## Next steps

- [Tooling dependencies](tooling-dependencies.md)
- [Security posture](security-posture.md)
- [Capabilities model](capabilities-model.md)
