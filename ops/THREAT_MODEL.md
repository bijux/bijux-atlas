# Ops Threat Model

- Owner: `bijux-atlas-operations`
- Purpose: `capture high-level threat categories against ops contracts, evidence, and execution surfaces`
- Consumers: `checks_ops_final_polish_contracts`
- Authority Tier: `tier2`
- Audience: `reviewers`

## Threat Categories

- Supply-chain tampering (action/image/tool drift)
- Contract drift and duplicate authority injection
- Generated evidence tampering or stale evidence replay
- Runtime artifact path escape into authored surfaces
- Documentation misdirection and stale redirect surfaces
- Human workflow bypass (missing sign-off, missing ownership, missing review)

## Mitigations

- Immutable pins and allowlists
- Schema and authority integrity checks
- Evidence lineage and completeness validation
- Workflow run isolation and artifact-root enforcement
- Human workflow maturity checks and sign-off contracts

## Residual Risk

- CI hosted-runner runtime behavior requires periodic execution proof beyond static checks.
