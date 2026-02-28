# Store integrity model

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7f82f1b0`
- Reason to exist: define immutability, integrity checks, and GC boundaries for serving-store data.

## Integrity guarantees

- Published release artifacts are immutable.
- Registry-to-artifact mapping is deterministic and auditable.
- Serving-store reads never mutate release source artifacts.

## Integrity checks

- Artifact hash verification during publish and load.
- Registry reference validation before alias progression.
- Serving-store schema/index checks during startup and release transitions.

## GC boundaries

- GC removes only unreachable and policy-expired artifacts.
- GC never removes currently aliased releases.
- GC actions are traceable through operator reports.

## Terminology used here

- Artifact: [Glossary](../glossary.md)
- Release: [Glossary](../glossary.md)

## Next steps

- [Storage](storage.md)
- [Registry federation model](registry-federation-model.md)
- [Operations retention and GC](../operations/retention-and-gc.md)
