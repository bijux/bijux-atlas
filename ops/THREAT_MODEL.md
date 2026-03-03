# Ops Threat Model

- Owner: `bijux-atlas-operations`
- Purpose: capture threat classes and controls for ops automation surfaces.
- Consumers: `checks_ops_final_polish_contracts`

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
