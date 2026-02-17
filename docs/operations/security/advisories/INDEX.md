# Security Advisories Index

- Owner: `bijux-atlas-security`

## What

Index page for advisory process and published security notices.

## Why

Advisories require a stable entrypoint for incident response and audits.

## Scope

Security advisories and publication workflow references.

## Non-goals

No operational runbook duplication.

## Contracts

- Advisory procedures must align with `operations/security/advisory-process.md`.

## Failure modes

Missing index prevents discoverability and creates response delays.

## How to verify

```bash
$ make docs
```

Expected output: docs checks pass.

## See also

- [Security Operations](../INDEX.md)
- [Advisory Process](../advisory-process.md)
- [Terms Glossary](../../../_style/terms-glossary.md)
