# Policies Docs Index

Policy philosophy:
- One source of truth for deploy/runtime safety guardrails.
- Deterministic validation with strict unknown-key rejection.
- No implicit defaults unless explicitly documented.

SSOT location:
- `configs/policy/policy.json`
- `configs/policy/policy.schema.json`

Docs:
- [Architecture](ARCHITECTURE.md)
- [Public API](PUBLIC_API.md)
- [Config schema](CONFIG_SCHEMA.md)
- [Schema](SCHEMA.md)
- [Effects policy](EFFECTS.md)
- [Schema evolution guide](SCHEMA_EVOLUTION.md)
- [Evolution](EVOLUTION.md)
- [Policy change checklist](CHANGE_CHECKLIST.md)

## API stability

Public API is defined only by `docs/PUBLIC_API.md`; all other symbols are internal and may change without notice.

## Invariants

Core invariants for this crate must remain true across releases and tests.

## Failure modes

Failure modes are documented and mapped to stable error handling behavior.

## How to extend

Additions must preserve crate boundaries, update `docs/PUBLIC_API.md`, and add targeted tests.

