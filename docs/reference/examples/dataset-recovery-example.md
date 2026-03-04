# Dataset Recovery Example

- Owner: `bijux-atlas-operations`
- Audience: `operator`
- Type: `guide`
- Stability: `stable`
- Reason to exist: show a recovery flow after dataset serving regression.

## Recovery flow

1. identify last known good dataset
2. rollback catalog pointer
3. re-run readiness checks
4. document incident evidence
