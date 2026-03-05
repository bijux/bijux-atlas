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
3. Build docs using `mkdocs build --strict`.
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
- Canonical published URL: `https://bijux.github.io/bijux-atlas/`
- `mkdocs.yml` canonical source:
  - `site_url: https://bijux.github.io/bijux-atlas/`
  - `use_directory_urls: true`

## Internal Link Safety

- Internal docs must not link through `github.com/.../blob/.../*.md`.
- Published markdown local links must resolve to in-tree files under `docs/`.
- These are enforced by docs contracts:
  - `DOC-078` `docs.links.no_internal_github_blob_links`
  - `DOC-079` `docs.links.published_local_links_resolve`

## CNAME Support

Custom domain support is optional.

1. Add `docs/CNAME` with the desired hostname.
2. Ensure the hostname is configured in GitHub Pages settings.
3. Keep `docs/CNAME` under version control so domain ownership remains auditable.
