# Common failure messages

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@2026-03-01`
- Reason to exist: translate recurring control-plane failures into direct next actions.

| Failure shape | Meaning | First action |
| --- | --- | --- |
| missing `--allow-subprocess` | command requires an effect gate you did not grant | rerun through the canonical make target or add the explicit flag |
| docs registry missing required field | canonical docs metadata drifted | run `make docs-registry` and inspect the changed source page |
| strict docs build warning | published docs link, nav, or file contract drifted | run `mkdocs build --strict` and fix the first warning, not the last |
| required contract skipped | lane selection or tooling availability is hiding a merge blocker | rerun the required lane and remove the skip condition |
| missing external tool | the selected command depends on a gated binary not present locally | run `make ops-tools-verify` and install only the missing tool |

## Verify success

```bash
make ci-pr
make docs-build
```

## Next steps

- [Debug failing checks](debug-failing-checks.md)
- [How to reproduce CI locally](reproduce-ci-locally.md)
- [Adding external tooling](adding-external-tooling.md)
