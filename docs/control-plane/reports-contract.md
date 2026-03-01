# Reports contract

- Owner: `platform`
- Type: `reference`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@331751e4`
- Reason to exist: define report schema expectations and versioning behavior.

## Contract guarantees

- Report schema is versioned.
- Required keys are stable for consumers in CI and local tooling.
- Report ordering is deterministic for stable diffs.

## Meaning of the core fields

- `schema_version`: compatibility contract for report readers.
- `summary` and status fields: gate outcome without log scraping.
- lane and context metadata: explains where the run happened and with which selection.
- artifact paths: point reviewers to the exact evidence written under `artifacts/`.

## Required report fields

- `schema_version`
- execution summary and status
- lane/context metadata
- artifact paths when applicable

## Strictness rules

- Missing required keys are a contract failure, not a warning.
- Consumers must key off report fields, never free-form terminal text.
- Version changes require an explicit compatibility review and coordinated consumer update.

## Schema versioning policy

- Increment `schema_version` only when consumers need an explicit compatibility signal.
- Additive fields keep backward compatibility when existing required fields remain stable.
- Breaking field removals or type changes require coordinated CI consumer updates before merge.

## Verify success

Run a report-producing command and inspect JSON output.

```bash
cargo run -q -p bijux-dev-atlas -- docs inventory --allow-write --allow-subprocess --format json
```

## Next steps

- [CI report consumption](ci-report-consumption.md)
- [Evidence writing style](evidence-writing-style.md)
- [Debug failing checks](debug-failing-checks.md)
