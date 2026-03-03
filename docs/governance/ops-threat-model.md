# Ops Threat Model

- Owner: `bijux-atlas-operations`
- Audience: `contributors`
- Scope: `operations governance`

Canonical threat modeling narrative for operations automation and control-plane surfaces.

## Threat Categories

- supply chain compromise
- artifact tampering
- unauthorized control-plane execution

## Mitigations

- digest pinning
- schema validation
- allowlisted command surfaces

## Residual Risk

Residual risk remains for compromised upstream registries and local machine trust.
