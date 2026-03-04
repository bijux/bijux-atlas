---
title: Docs Site Deployment
audience: operator
type: runbook
stability: stable
owner: docs-governance
last_reviewed: 2026-03-04
tags:
  - docs
  - deployment
  - github-pages
---

# Docs Site Deployment

This repository publishes the documentation site to GitHub Pages from `.github/workflows/docs-deploy.yml`.

## Triggering Rules

- Automatic on push to `main`.
- Automatic on version tags matching `v*`.
- Manual via `workflow_dispatch` for emergency rebuilds.

## Deployment Pipeline

1. Validate `mkdocs.yml` `site_dir` is `artifacts/docs/site`.
2. Install pinned docs dependencies from `configs/docs/requirements.lock.txt` and `configs/docs/package-lock.json`.
3. Build docs using `bijux-dev-atlas` (`docs build`) in strict mode.
4. Upload `artifacts/docs/site` using `actions/upload-pages-artifact`.
5. Deploy with `actions/deploy-pages`.

## Required Repository Configuration

GitHub repository Pages must be configured to deploy from **GitHub Actions**.
The governance source of truth is:

- `docs/_internal/governance/docs-pages-repository-settings.md`

## Output Contract

- Built site directory: `artifacts/docs/site`
- Pages environment: `github-pages`
- Deployment URL: provided by GitHub Pages deployment output
