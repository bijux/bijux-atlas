# Schema Version Bump Playbook

Use this playbook when changing any versioned schema under `packages/atlasctl/src/atlasctl/contracts/schema/schemas/` or `ops/schema/`.

## Required steps

1. Add a new versioned schema file (do not edit old versions in-place).
2. Update schema catalog/registry references.
3. Update any producer commands to emit the new schema version.
4. Update validators/checks and goldens/samples.
5. Update `packages/atlasctl/docs/release-notes.md` with the schema change.
6. Run the schema contract checks and regenerate registry/docs artifacts if needed.

## Verification

- `./bin/atlasctl check run --group contracts`
- `./bin/atlasctl check run --id checks_docs_ci_lane_mapping` (if docs changed)
