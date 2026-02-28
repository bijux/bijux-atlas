# Registry federation model

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7f82f1b0`
- Reason to exist: define deterministic registry merge and conflict handling.

## Federation behavior

- Registry inputs merge deterministically for equivalent source sets.
- Conflicts are explicit and block release alias updates.
- Merge outcomes are reproducible across local, CI, and release lanes.

## Conflict resolution rules

- Prefer canonical version ordering and explicit compatibility policy.
- Reject ambiguous ownership or alias collisions.
- Require explicit operator/contributor action before progression.

## Terminology used here

- Registry: [Glossary](../glossary.md)
- Release: [Glossary](../glossary.md)

## Next steps

- [Dataflow](dataflow.md)
- [Store integrity model](store-integrity-model.md)
- [Runbook: registry federation](../operations/runbooks/registry-federation.md)
