# Contract coverage dashboard

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@2026-03-01`
- Reason to exist: explain how to interpret contract coverage artifacts without reading raw dumps first.

## What it shows

- which contract groups ran
- how many tests passed, failed, skipped, or errored
- where machine-readable evidence was written
- which lanes are covered by the current contract set

## Where the artifact lives

- contributor-facing entrypoint: [Docs dashboard](../governance/docs-dashboard.md)
- machine artifact: `docs/_generated/docs-contract-coverage.json`

## How to read it

- start with the domain or group summary
- compare pass/fail totals before opening detailed artifacts
- treat skipped merge-blocking contracts as a failure in review, even if the raw run did not crash

## Verify success

```bash
make docs-registry
```

## Next steps

- [Reports contract](reports-contract.md)
- [Common failure messages](common-failure-messages.md)
- [Debug failing checks](debug-failing-checks.md)
