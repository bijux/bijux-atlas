# Contracts model

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7f82f1b0`
- Reason to exist: define contract types, versioning, and diffing expectations.

## Contract types

- Runtime behavior contracts (query, API, errors, pagination)
- Artifact and schema contracts
- Registry and compatibility contracts
- Control-plane reporting contracts

## Versioning and diffing

- Contract versions are explicit and tracked.
- Diffs classify additive, compatible, and breaking changes.
- Breaking changes require approval and migration communication.

## Terminology used here

- Contract: [Glossary](../glossary.md)
- Stability: [Glossary](../glossary.md)

## Next steps

- [Reference contracts](../reference/contracts/index.md)
- [Dataflow](dataflow.md)
- [Development control plane](../development/control-plane.md)
