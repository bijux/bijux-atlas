# ADR-0004: Plugin Contract And Umbrella Dispatch

Status: Accepted

Context:
Atlas must integrate into the Bijux umbrella CLI while remaining independently versioned.

Decision:
- Standardize plugin metadata handshake and binary naming (`bijux-atlas`).
- Keep command surface in `atlas ...` namespace.
- Enforce compatibility range checks.

Consequences:
- Predictable ecosystem composition.
- Contract drift requires explicit docs/tests updates.
