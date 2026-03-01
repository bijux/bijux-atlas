# Lane matrix

- Owner: `platform`
- Type: `reference`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@2026-03-01`
- Reason to exist: define the durable local, PR, nightly, and release lane contract for control-plane workflows.

| Lane | Primary wrapper | Core command | Goal | Cost profile |
| --- | --- | --- | --- | --- |
| `local` | `make ci-fast` | `check run --suite ci_fast --format json` | short feedback loop before pushing | cheap |
| `pr` | `make ci-pr` | `check run --suite ci_pr --format json` | merge-blocking pull-request validation | medium |
| `nightly` | `make ci-nightly` | `check run --suite ci_nightly --include-internal --include-slow --format json` | broad regression and expensive checks | expensive |
| `release` | `make contracts-release` | `cargo run -q -p bijux-dev-atlas -- contracts ops --lane release --profile ci --required --format json` | release-readiness evidence and contract enforcement | expensive |

## Selection rules

- Add low-cost checks to `ci_fast` only when they preserve rapid local feedback.
- Put required merge guards in `ci_pr`.
- Put broad or environment-heavy checks in `ci_nightly` or release lanes.
- Release lanes must prefer evidence-bearing contract runs over ad-hoc shell scripts.

## Verify success

```bash
make ci-fast
make ci-pr
make ci-nightly
```

## Next steps

- [How suites work](how-suites-work.md)
- [Performance budget](performance-budget.md)
- [How to reproduce CI locally](reproduce-ci-locally.md)
