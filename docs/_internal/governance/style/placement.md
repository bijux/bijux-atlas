# Placement Decision Tree

1. Is this end-user framing, scope, or promise?
   Place in `docs/product/`.
2. Is this first-run setup or onboarding flow?
   Place in `docs/operations/` and reference from `docs/start-here.md`.
3. Is this stable API/artifact/schema contract?
   Place in `docs/reference/contracts/`.
4. Is this operational procedure, incident response, or deployment runbook?
   Place in `docs/operations/`.
5. Is this contributor workflow (CI, make, cargo, release process)?
   Place in `docs/development/`.
6. Is this architecture boundary, crate interaction, or design map?
   Place in `docs/architecture/`.
7. Is this immutable design decision record?
   Place in `docs/adrs/`.
8. Is this technical reference material not in contract scope?
   Place in `docs/reference/`.

If a document spans multiple categories, split it and cross-link.
