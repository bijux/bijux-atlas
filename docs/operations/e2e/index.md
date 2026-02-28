# End-to-end Tests

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@c59da0bf`
- Reason to exist: define the exact end-to-end test entrypoint and expected outputs.

## Commands

```bash
make k8s-validate
make ops-k8s-tests
```

## Expected outputs

- Validation and test summaries in command output.
- Evidence artifacts under `artifacts/evidence/k8s/`.
- No failing checks in release gate reports.

## Core guides

- [Kubernetes tests](k8s-tests.md)
- [Fixture taxonomy](fixtures.md)

## Next

- [Kubernetes Operations](../k8s/index.md)
- [Load Testing](../load/index.md)
