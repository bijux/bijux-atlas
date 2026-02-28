# Release Workflow

Owner: `bijux-atlas-operations`  
Audience: `operator`  
Type: `runbook`  
Reason to exist: define canonical release update and rollback operations.

## Update

```bash
make ops-release-update
```

## Rollback

```bash
make stack-down
```

Artifacts remain immutable; rollback only changes catalog pointers.
