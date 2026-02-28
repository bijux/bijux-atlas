# Depth Rubric

## What
This rubric defines measurable documentation depth requirements used by automated lint checks.

## Why
Documentation must be executable and reviewable. Surface-level text without contracts, examples, failure behavior, and verification commands is rejected.

## Contracts
- Required sections for reference and contract pages:
  - `## What`
  - `## Why`
  - `## Contracts`
  - `## Failure Modes`
  - `## How to Verify`
- Required sections for runbooks:
  - `## Symptoms`
  - `## Metrics`
  - `## Commands`
  - `## Mitigations`
  - `## Rollback`
- Example minimums:
  - Reference/Contracts/Operations docs (non-index): at least `1` fenced code block.
- Verification minimums:
  - Must contain a verify section with at least one runnable command block.
- Architecture diagrams:
  - Each major architecture page must include at least one `mermaid` block or an image under `docs/_assets/diagrams/`.
- Anti-handwavy language:
  - Forbidden terms: `simple`, `just`, `obvious`, `etc`.

## Failure Modes
- Missing required sections reduce maintainability and create hidden behavior.
- Missing verification steps blocks reproducibility during incidents.
- Missing diagrams increases architecture ambiguity.

## How to Verify
```bash
bijux dev atlas docs check --report text
```

See also:
- [Depth policy](depth-policy.md)
- [Structure Templates](structure-templates.md)
- [Docs style](../docs-style.md)
