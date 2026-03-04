# Audit And Security Event Classification

## Audit Logging Requirements

Security-relevant operations must emit audit events with:

- event identifier
- event class
- principal identifier
- target resource
- outcome
- timestamp

## Security Event Classes

- `auth.success`
- `auth.failure`
- `authorization.denied`
- `credential.issued`
- `credential.rotated`
- `policy.validation_failure`
- `integrity.violation`
- `security.configuration_error`
- `recovery.failover`

## Classification Rules

- classify by security control that detected or enforced the action
- use stable class names for alerting and reporting
- keep event payloads free of raw secret values
