# Start Here

Owner: `platform`  
Audience: `user`, `operator`, `contributor`  
Type: `guide`  
Reason to exist: provide one fast path from clone to verified local Atlas behavior.

## What Atlas Is

Atlas is the stable platform surface for operating, evolving, and consuming the bijux ecosystem through explicit contracts, predictable workflows, and verifiable runtime behavior.

## Five Minute Quickstart

1. Validate local prerequisites:
   - `make ops-prereqs`
   - `make ops-doctor`
2. Run the canonical local stack:
   - `make ops-local-full`
3. Verify service health:
   - `curl -fsS http://127.0.0.1:8080/healthz`
4. Run one API smoke query:
   - `curl -fsS 'http://127.0.0.1:8080/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=1'`
5. Review run artifacts:
   - `make ops-artifacts-open`

## Canonical Workflow Details

- [Run Locally](operations/run-locally.md)
- [Deploy](operations/deploy.md)
- [API Quick Reference](api/quick-reference.md)

## What To Read Next

- [Product](product/index.md)
- [API](api/index.md)
- [Operations](operations/index.md)
- [Development](development/index.md)
- [Reference](reference/index.md)
