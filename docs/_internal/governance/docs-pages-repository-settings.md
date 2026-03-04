---
title: Docs Pages Repository Settings
audience: internal
type: policy
stability: stable
owner: docs-governance
last_reviewed: 2026-03-04
tags:
  - docs
  - governance
  - github-pages
---

# Docs Pages Repository Settings

## Required Settings

- Repository Pages source must be set to **GitHub Actions**.
- The deployment workflow is `.github/workflows/docs-deploy.yml`.
- The deployment environment is `github-pages`.

## Docs Release Policy

- Docs deployment runs on every push to `main`.
- Docs deployment runs on release tags matching `v*`.
- Manual deployment is allowed through `workflow_dispatch` for rebuild and recovery.
- `mkdocs` output location is fixed to `artifacts/docs/site`.
- Release artifacts must come from the same `site_dir` configured in `mkdocs.yml`.

## Why This Exists

The docs pipeline uploads `artifacts/docs/site` as a Pages artifact and then deploys using `actions/deploy-pages`.
If repository settings are changed away from GitHub Actions, docs publication fails even when the workflow is green.

## Verification

Use the docs deployment runbook:

- `docs/operations/docs-site-deploy.md`
