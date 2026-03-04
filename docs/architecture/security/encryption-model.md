# Encryption Model

- Owner: `bijux-atlas-security`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Reason to exist: define transport and at-rest encryption behavior.

## Transport Encryption

- TLS is required in production ingress paths.
- Minimum accepted TLS version is controlled by runtime security config.
- Requests may be rejected when secure transport is required and not present.

## At-Rest Encryption

- Data-protection abstraction supports encryption/decryption operations.
- Dataset encryption can be enabled as a governed option for artifact materialization.
- Encryption is paired with integrity checks before serving cached artifacts.
