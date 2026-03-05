---
title: Package Release Workflow
audience: contributor
type: guide
stability: stable
owner: api-contracts
last_reviewed: 2026-03-05
tags:
  - release
  - python
  - sdk
---

# Package Release Workflow

Release the Python SDK from `packages/bijux-atlas-python`.

1. Verify package boundaries and metadata:
   - `bijux-dev-atlas packages verify --format json`
2. Verify generated client docs and examples:
   - `bijux-dev-atlas clients verify --client atlas-client --format json`
3. Verify Python test suite via dev-atlas:
   - `bijux-dev-atlas clients python test --client atlas-client --format json`
4. Build release artifacts:
   - `python -m build packages/bijux-atlas-python`
5. Publish to PyPI from CI using signed artifacts.

Release readiness requires a passing governance report section for:

- `Packages boundary compliance`
- `Clients tooling purity`
- `Directory Purity`
