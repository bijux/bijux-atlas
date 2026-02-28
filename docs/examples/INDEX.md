# Examples Index

- Owner: `bijux-atlas-docs`

## What
Canonical runnable examples for policy configuration and catalog structures.

## Why
Examples are treated as executable interfaces and validated by docs hardening gates.

## Scope
Covers curated examples under `docs/examples/` only.

## Non-goals
Does not duplicate full API usage guides or schema references.

## Contracts
- Examples must be deterministic.
- Shell snippets intended for execution must declare `# blessed-snippet`.
- Config examples must validate against contract schemas.

## Failure modes
Outdated examples can mislead operators and break onboarding workflows.

## How to verify
```bash
$ make docs
```

Expected output: example extraction, snippet execution, and schema checks pass.

## See also
- [API Quick Reference](../api/quick-reference.md)
- [Run Locally](../operations/run-locally.md)
- [Repo Surface](../development/repo-surface.md)
