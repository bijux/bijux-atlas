# Suite Membership Policy

- Owner: `team:atlas-governance`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Reason to exist: define how a governed entry moves between checks and contracts without losing history.

## Membership boundary

Checks are quality gates. Contracts are governance invariants. An entry belongs to one governing suite by default, even when one surface internally invokes the other.

## Allowed overlap

Use `overlaps_with[]` only to document a deliberate conceptual overlap. It does not create dual suite membership. A check may reference a contract-backed workflow, and a contract may rely on lower-level checks, but the SSOT registry entry stays singular unless governance explicitly records a different target id in `overlaps_with[]`.

## How to move an entry

1. Add the successor entry to the destination registry with a new stable id.
2. Record the old id in the deprecations registry before changing suite files.
3. Update the suite file so membership is explicit and singular.
4. Update reference docs and the validation entrypoints page in the same change.
5. Update the suite baseline only when the shrink is intentional and reviewed.
