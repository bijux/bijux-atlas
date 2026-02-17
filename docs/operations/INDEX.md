# Operations Index

- Owner: `bijux-atlas-operations`

## What

Canonical entrypoint for operating atlas in production.

## Why

Provides one stable operations surface linking deployment, observability, load, runbooks, and security.

## Scope

Kubernetes operations, observability posture, load validation, incident runbooks, and security practices.

## Non-goals

Does not define product semantics or internal crate APIs.

## Contracts

- [Kubernetes](k8s/INDEX.md)
- [Observability](observability/SLO.md)
- [Load](load/k6.md)
- [Runbooks](runbooks/INDEX.md)
- [Security](security/SECURITY_POSTURE.md)

## Failure modes

Missing operational references causes inconsistent incident response and unsafe deployments.

## How to verify

```bash
$ make docs
```

Expected output: operations links resolve and docs checks pass.

## See also

- [Product SLO Targets](../product/SLO_TARGETS.md)
- [Contracts Metrics](../contracts/metrics.md)
- [Terms Glossary](../_style/TERMS_GLOSSARY.md)
