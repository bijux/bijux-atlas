# Ingest and QC model

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7f82f1b0`
- Reason to exist: define ingest validation contracts and quality-control failure behavior.

## Validation contract

- Ingest accepts only schema-valid and policy-compliant inputs.
- QC checks run before artifact publication.
- Any validation or QC failure blocks release progression.

## Common failures

- Missing required fields in source datasets.
- Contract version mismatch.
- Quality thresholds not met for release publication.

## Terminology used here

- Fixture: [Glossary](../glossary.md)
- Artifact: [Glossary](../glossary.md)

## Next steps

- [Dataflow](dataflow.md)
- [Error model](error-model.md)
- [Runbook: dataset corruption](../operations/runbooks/dataset-corruption.md)
