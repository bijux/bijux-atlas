# Compatibility And Deprecation Process

- Owner: `bijux-atlas-governance`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Reason to exist: provide one SSOT workflow for governed renames, deprecations, breaking changes, and release communication.

## Required sequence

1. Add or update the governed entry in `configs/governance/deprecations.yaml`.
2. Keep both old and new surfaces accepted during the overlap window.
3. Add redirects when the surface is a reader URL.
4. Add migration notes when the surface is a governed report schema.
5. Run `governance deprecations validate`, `governance breaking validate`, and `governance doctor`.
6. Regenerate release evidence before a release candidate.

## Acceptance rules

- Production profiles cannot keep deprecated keys unless an active exception covers `OPS-COMP-001`.
- Breaking changes require `ops/release/notes/breaking.md` entries.
- Chart-breaking changes require a chart major version that satisfies the governed policy.
- Evidence bundles must carry the governance doctor report and institutional delta.
