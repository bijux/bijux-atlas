# Error model

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7f82f1b0`
- Reason to exist: define error categorization and propagation expectations across runtime layers.

## Categories

- Validation errors: input or contract violations.
- Dependency errors: unavailable storage, registry, or external runtime dependency.
- Internal errors: invariant or implementation failures.
- Overload errors: capacity protections and explicit degradation responses.

## Propagation rules

- Keep category and cause explicit across layer boundaries.
- Preserve deterministic API status mapping for equivalent failures.
- Attach repro context in control-plane reports for actionable triage.

## Examples

- Invalid ingest payload -> validation error -> publish blocked.
- Store unavailable -> dependency error -> query degradation path.
- Unsupported contract version -> validation error -> release blocked.

## Terminology used here

- Contract: [Glossary](../glossary.md)
- Release: [Glossary](../glossary.md)

## Next steps

- [Reference errors](../reference/errors.md)
- [Dataflow](dataflow.md)
- [Common failure catalog](common-failure-catalog.md)
