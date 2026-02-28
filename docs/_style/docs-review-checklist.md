# Docs Review Checklist

## What
Template for pull requests that modify documentation surfaces.

## Why
Consistent review criteria prevents drift and keeps contracts enforceable.

## Contracts
- [ ] File names follow `kebab-case.md` or `INDEX.md`.
- [ ] Page includes required depth sections from [`DEPTH_RUBRIC.md`](DEPTH_RUBRIC.md).
- [ ] Examples are runnable and match bijux dev atlas commands in repo.
- [ ] Failure behavior is explicit and testable.
- [ ] Verify section includes executable commands.
- [ ] Terms align with glossary.
- [ ] SSOT references link to `docs/contracts/*` instead of copied tables.
- [ ] If architecture behavior changed, diagram is updated.

## Failure Modes
- Missing checklist updates allow undocumented surface changes.

## How to Verify
```bash
make docs
```

See also:
- [Depth Rubric](DEPTH_RUBRIC.md)
- [Writing Rules](writing-rules.md)
- [Structure Templates](structure-templates.md)
