# Release Lane Guarantees

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define what each release lane must prove before promotion.

## Release lanes

| Lane | Required evidence | Blocking guarantee |
| --- | --- | --- |
| `ci_pr` | contract reports, tests, docs preview | no merge with failing repo, runtime, docs, or config contracts |
| `ci_nightly` | regenerated docs/contracts, broader static validation | no silent drift across generated policy and docs surfaces |
| `release` | readiness scorecard, release contracts, operator verification | no promotion without explicit release-readiness proof |

## What operators should expect

- Every lane writes artifacts under `artifacts/<run-id>/`.
- A failed lane means promotion is blocked until the failing contract is fixed or intentionally changed.
- Release promotion depends on the strongest lane, not the fastest lane.

## Verify success

```bash
make ops-readiness-scorecard
make contracts-release
```

Expected result: both commands emit successful release-readiness evidence.

## Next

- [Quality Wall](quality-wall.md)
- [Release Workflow](../release-workflow.md)
- [Rollback Procedure](rollback-procedure.md)
