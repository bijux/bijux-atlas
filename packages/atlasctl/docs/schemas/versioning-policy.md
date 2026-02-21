# Schema Versioning Policy

`atlasctl` schema IDs use the form `atlasctl.<name>.v<major>`.

Rules:

- `catalog.json` is the source of truth for schema IDs, versions, and files.
- Schema IDs must include a major version suffix (for example `atlasctl.help.v1`).
- `version` in catalog must match the suffix number in `name`.
- Breaking changes require a new major version and a new schema file.
- Existing schema versions are immutable after release.
- Every schema in catalog must have a sample payload in `tests/goldens/samples/`.
- Any schema change must include a release-note entry in `packages/atlasctl/docs/release-notes.md`.
- `packages/atlasctl/src/atlasctl/contracts/schema/schemas/README.md` is generated and must be kept in sync with schema files.
