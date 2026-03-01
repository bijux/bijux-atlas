# Depth Policy

- Owner: `docs-governance`

## What

Defines minimum and maximum detail depth by documentation type.

## Contracts

- Product docs: concise contract-level depth (1 page each concept).
- Reference docs: medium depth with precise semantics and limits.
- Runbooks: high depth with executable commands and expected outputs.
- ADRs: decision context + alternatives + consequences.
- Generated docs: machine-derived, no manual expansion.

## Enforcement

Depth is enforced by templates and docs lint gates; do not add ad-hoc narrative pages.

## See also

- [Structure Templates](structure-templates.md)
- [Docs style](../docs-style.md)
- [Glossary](../../../glossary.md)
