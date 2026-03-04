# Data Classification Model

- Owner: `bijux-atlas-security`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: define sensitivity classes and handling expectations.

## Classes

- public: safe to expose by default
- internal: controlled operational metadata
- sensitive: redaction required, least-privilege access
- secret: never exposed in logs or public outputs

## Source Of Truth

- `configs/security/data-classification.yaml`
- `configs/security/data-protection.yaml`
