# Contributor onboarding rubric (30 minutes)

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@bdd91bc0`
- Reason to exist: provide a bounded onboarding rubric that proves a contributor can read, run, and extend Atlas workflows.

## Checklist

- [ ] Read [Start here](../start-here.md) and confirm the three audience paths.
- [ ] Run local golden path: [Run locally (5 minutes)](../operations/run-locally.md).
- [ ] Read architecture flow: [Dataflow](../architecture/dataflow.md) and [Boundaries](../architecture/boundaries.md).
- [ ] Read contributor workflow: [Development](index.md) and [Control-plane](../control-plane/index.md).
- [ ] Run contributor checks:

```bash
make check
```

- [ ] Read factual surfaces: [Reference](../reference/index.md) and [API](../api/index.md).
- [ ] Confirm core terminology in [Glossary](../glossary.md).

## Verify success

A new contributor can explain runtime flow, run the local stack, locate canonical references, and execute baseline checks without relying on governance internals.

## Next steps

- [Contributing](contributing.md)
- [How to change docs](how-to-change-docs.md)
- [Where truth lives](where-truth-lives.md)
