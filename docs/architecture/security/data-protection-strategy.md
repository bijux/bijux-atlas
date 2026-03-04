# Data Protection Strategy

- Owner: `bijux-atlas-security`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Reason to exist: define confidentiality and integrity controls for data paths.

## Strategy

Atlas data protection uses defense-in-depth with explicit controls for:

- transport confidentiality (TLS)
- at-rest encryption abstractions
- artifact and manifest integrity verification
- tamper detection and corruption quarantine

## Security Goals

- prevent unauthorized data disclosure
- detect integrity violations before serving data
- preserve forensic evidence through security event retention

## Contract Source

- `configs/security/data-protection.yaml`
