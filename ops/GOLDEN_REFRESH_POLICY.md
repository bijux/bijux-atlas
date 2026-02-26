# Golden Refresh Policy

- Authority Tier: `machine`
- Audience: `contributors`
## Scope
- Applies to committed golden snapshots, curated report examples, and generated example evidence under `ops/_generated.example/` and test goldens under `crates/**/tests/goldens/`.

## Regeneration
- Regenerate goldens only through canonical `bijux dev atlas ...` commands or documented `make` wrappers.
- Include the exact regeneration command(s) in the commit message body or PR description.
- Refresh only the affected golden set; avoid unrelated snapshot churn.

## Review
- Reviewers must inspect semantic diffs, not only hash/size changes.
- Any schema-affecting golden refresh must reference the corresponding schema or contract change.
- Drift caused by renamed or retired paths must include migration notes with cutoff dates.

## Approval
- Golden refresh changes require approval from the owning domain reviewer.
- Cross-domain golden refresh changes require at least one reviewer from each affected domain.
