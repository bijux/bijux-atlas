# Policies Docs Index

Policy philosophy:
- One source of truth for deploy/runtime safety guardrails.
- Deterministic validation with strict unknown-key rejection.
- No implicit defaults unless explicitly documented.

SSOT location:
- `configs/policy/policy.json`
- `configs/policy/policy.schema.json`

Docs:
- [Architecture](architecture.md)
- [Public API](public-api.md)
- [Config schema](config-schema.md)
- [Schema](schema.md)
- [Effects policy](effects.md)
- [Schema evolution guide](schema-evolution.md)
- [Evolution](evolution.md)
- [Policy change checklist](change-checklist.md)
- [Policy authoring guide](policy-authoring-guide.md)

- [How to test](testing.md)
- [How to extend](#how-to-extend)

## API stability

Public API is defined only by `docs/public-api.md`; all other symbols are internal and may change without notice.

## Invariants

Core invariants for this crate must remain true across releases and tests.

## Failure modes

Failure modes are documented and mapped to stable error handling behavior.

## How to extend

Additions must preserve crate boundaries, update `docs/public-api.md`, and add targeted tests.

- Central docs index: docs/index.md
- Crate README: ../README.md
