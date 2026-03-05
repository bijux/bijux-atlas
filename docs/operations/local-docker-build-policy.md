---
title: Local docker build policy
audience: operators
type: policy
stability: stable
owner: platform
last_reviewed: 2026-03-04
tags:
  - operations
  - docker
related:
  - docs/operations/docker-build-network-policy.md
---

# Local docker build policy

- Local image build must not require repository secrets.
- Build inputs must be available from repository sources and declared base images.
- Networked downloads during build are restricted by governed build network policy.

## Local build command

```bash
docker build --pull=false -f docker/images/runtime/Dockerfile -t bijux-atlas:local .
```


Enforcement reference: OPS-ROOT-023.
