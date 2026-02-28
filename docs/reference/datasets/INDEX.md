# Datasets Reference Index

- Owner: `bijux-atlas-store`

## What

Reference entrypoint for dataset lifecycle operations.

## Why

Centralizes publication and rollback procedures.

## Scope

Lifecycle, promotion, rollback, mirroring, offline mode.

## Non-goals

No API endpoint behavior details.

## Contracts

- [Lifecycle](lifecycle.md)
- [Promotion](promotion.md)
- [Rollback](rollback.md)
- [Mirroring](mirroring.md)
- [Offline Mode](offline-mode.md)

## Failure modes

Using stale lifecycle procedures can break immutability and availability.

## How to verify

```bash
$ make docs
```

Expected output: all linked references render with no broken links.

## See also

- [Store Reference](../store/INDEX.md)
- [Registry Reference](../registry/INDEX.md)
- [Incident Playbook](../../operations/runbooks/incident-playbook.md)
