# Config Versioning

- Owner: `docs-governance`
- Stability: `stable`

## What

Versioning policy for config schemas and runtime config contracts.

## Why

Prevents silent breaking changes in policy/config surfaces consumed by CI and runtime.

## Scope

`configs/policy/policy.schema.json`, `configs/policy/policy.json`, `configs/contracts/env.schema.json`, `docs/contracts/CONFIG_KEYS.json`, and mirrored contract docs.

## Non-goals

Does not define API endpoint versioning.

## Contracts

- Every schema carries an explicit version field (`schema_version` or equivalent).
- Breaking schema changes require a version bump and compatibility note.
- `configs/policy/policy.schema.json` must stay in sync with `docs/contracts/POLICY_SCHEMA.json`.

## Failure modes

- Unversioned schema edits break downstream config loaders.
- Drift between configs and contract docs causes CI/runtime mismatch.

## How to verify

```bash
$ make config-validate
$ make config-drift
```

Expected output: version policy checks and drift checks pass.

## See also

- [Config Schema Versioning](../reference/registry/config-schema-versioning.md)
- [Config Changelog](config-changelog.md)
- `configs/README.md`
- [Terms Glossary](../_style/terms-glossary.md)
