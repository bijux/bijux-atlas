# Contributing

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@d489531b`
- Reason to exist: define merge requirements and review standards.

## Contribution Checklist

1. Reproduce and scope change intent before editing.
2. Update the canonical doc page when behavior changes.
3. Run required validation commands.
4. Keep changes small and logically grouped.
5. Include clear, durable commit messages.

## Required Validation

```bash
make check
make docs
make test
```

## Code Review Standards

- Contracts and policy changes must be explicit.
- Surface changes require matching docs/reference updates.
- No hidden behavior behind script-only entrypoints.

## Verify Success

All required checks pass and documentation reflects behavior truthfully.

## What to Read Next

- [Control-plane](../control-plane/index.md)
- [Doc Conventions](doc-conventions.md)
- [How to Change Docs](how-to-change-docs.md)
- [Contributor onboarding rubric (30 minutes)](contributor-onboarding-rubric.md)

## Document Taxonomy

- Audience: `contributor`
- Type: `guide`
- Stability: `stable`
- Owner: `platform`
