# Release Workflow

Owner: `bijux-atlas-operations`  
Audience: `operator`  
Type: `runbook`  
Reason to exist: define canonical release update and rollback operations.

## Update

```bash
make ops-release-update DATASET=medium
```

## Rollback

```bash
make ops-release-rollback DATASET=medium
```

Artifacts remain immutable; rollback only changes catalog pointers.
