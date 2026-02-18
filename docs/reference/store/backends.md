# Backends

- Owner: `bijux-atlas-store`

Supported backends: local filesystem, HTTP readonly, and S3-like object store.

## What

`bijux-atlas` store backends implement one contract with different transport semantics.

- `localfs`: read/write, used for publish + local development.
- `http readonly`: read-only catalog/artifact fetch, never publishes.
- `s3/minio`: object-store fetch and publish paths with retry + checksum validation.

## Why

Enables consistent serving behavior across local, air-gapped, and object-storage deployments.

## Scope

Applies to catalog, manifest, and sqlite artifact access.

## Non-goals

Does not define identity/auth policy for external infrastructure.

## Contracts

- All backends must validate catalog shape before serving.
- Optional `catalog.sha256` sidecar is supported; when present, serve-time checksum must match.
- `http readonly` backend must reject publish/lock operations.
- `localfs` publish path is immutable per dataset identity (`release/species/assembly`).
- `s3/minio` behavior must be deterministic with retry budgets and no path traversal.

## Failure modes

- Catalog parse/validation failure -> backend rejects refresh.
- Checksum mismatch (`catalog.sha256`) -> backend rejects refresh.
- Network failures -> retried per backend policy, then surfaced as readiness/cache errors.

## How to verify

```bash
$ make ops-catalog-validate
```

Expected output: catalog checks pass for configured backend profile.

## See also

- [Reference Index](INDEX.md)
- [Contracts Index](../../contracts/contracts-index.md)
- [Terms Glossary](../../_style/terms-glossary.md)
