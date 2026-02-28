# Navigation Policy

- Owner: `docs-governance`

The documentation navigation is explicit and stable by design.

## Rules

- Navigation is defined only in `mkdocs.yml`.
- Automatic page discovery is intentionally not used.
- Top-level navigation labels are fixed: `Start Here`, `Product`, `Quickstart`, `Reference`, `Contracts`, `API`, `Operations`, `Development`, `Architecture`, `Science`, `Generated`, `ADRs`.
- Top-level navigation order is fixed: `Start Here -> Product -> Quickstart -> Reference -> Contracts -> API -> Operations -> Development -> Architecture -> Science -> Generated -> ADRs`.
- Structural changes must update nav in the same change.

## What

Section overview.

## Why

Rationale for this section.

## Scope

Scope of documents in this section.

## Non-goals

Out-of-scope topics for this section.

## Contracts

Normative links and rules.

## Failure modes

Failure modes if docs drift.

## How to verify

```bash
$ make docs
```

Expected output: docs checks pass.

## See also

- [Docs Home](../index.md)
- [Naming Standard](../_style/naming-standard.md)
- [Terms Glossary](../_style/terms-glossary.md)
