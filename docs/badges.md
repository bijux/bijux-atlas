---
title: Badge Catalog
audience: maintainer
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-12
---

# Badge Catalog

`docs/badges.md` is the shared badge catalog for Atlas README and documentation
surfaces.

Use this page as the reference point when badge blocks change so the repository
root, docs landing, and maintainer-facing surfaces stay consistent about what
Atlas publishes and how those surfaces link out.

Do not add one-off badge styles or ad hoc badge destinations in public-facing
surfaces. Reuse the catalog patterns here so repository, runtime, and
maintainer badges keep one contract.

Atlas is Rust-first, so the catalog focuses on:

- repository workflow badges
- crates.io and docs.rs badges for the public runtime crate
- GHCR package publication badges for released runtime surfaces
- documentation badges for the canonical `bijux-atlas*` handbooks
- maintainer summary badges for repository governance and docs delivery

## Repository Summary

<!-- bijux-atlas-badges:repository-summary:start -->
[![Rust 1.86+](https://img.shields.io/badge/rust-1.86%2B-DEA584?logo=rust&logoColor=white)](https://crates.io/crates/bijux-atlas)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-atlas/blob/main/LICENSE)
[![CI](https://github.com/bijux/bijux-atlas/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/ci.yml?query=branch%3Amain)
[![Docs](https://github.com/bijux/bijux-atlas/workflows/deploy-docs/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/deploy-docs.yml)
[![Crates Publish](https://github.com/bijux/bijux-atlas/workflows/release-crates/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/release-crates.yml)
[![GHCR Publish](https://github.com/bijux/bijux-atlas/workflows/release-ghcr/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/release-ghcr.yml)
[![GitHub Release](https://github.com/bijux/bijux-atlas/workflows/release-github/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/release-github.yml)
[![Release](https://img.shields.io/github/v/release/bijux/bijux-atlas?display_name=tag&label=release)](https://github.com/bijux/bijux-atlas/releases)
[![GHCR packages](https://img.shields.io/badge/ghcr-1%20package-181717?logo=github)](https://github.com/bijux?tab=packages&repo_name=bijux-atlas)
[![Published packages](https://img.shields.io/badge/published%20packages-1-2563EB)](https://github.com/bijux/bijux-atlas/tree/main/crates)
<!-- bijux-atlas-badges:repository-summary:end -->

## Runtime Package Summary

<!-- bijux-atlas-badges:runtime-package-summary:start -->
[![Rust 1.86+](https://img.shields.io/badge/rust-1.86%2B-DEA584?logo=rust&logoColor=white)]({{ crate_registry_url }})
[![crates.io](https://img.shields.io/crates/v/{{ crate_name }}?logo=rust)]({{ crate_registry_url }})
[![rust-docs](https://img.shields.io/badge/rust--docs-{{ crate_badge_label }}-DEA584?logo=rust&logoColor=white)]({{ crate_docs_url }})
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-atlas/blob/main/LICENSE)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--atlas-181717?logo=github)](https://github.com/bijux/bijux-atlas)
<!-- bijux-atlas-badges:runtime-package-summary:end -->

## Maintainer Summary

<!-- bijux-atlas-badges:maintainer-summary:start -->
[![Rust 1.86+](https://img.shields.io/badge/rust-1.86%2B-DEA584?logo=rust&logoColor=white)](https://github.com/bijux/bijux-atlas)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-atlas/blob/main/LICENSE)
[![Docs](https://github.com/bijux/bijux-atlas/workflows/deploy-docs/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/deploy-docs.yml)
[![Docs Audit](https://github.com/bijux/bijux-atlas/workflows/docs-audit/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/docs-audit.yml)
[![Ops Validate](https://github.com/bijux/bijux-atlas/workflows/ops-validate/badge.svg)](https://github.com/bijux/bijux-atlas/actions/workflows/ops-validate.yml)
<!-- bijux-atlas-badges:maintainer-summary:end -->

## Runtime Crate Badge

<!-- bijux-atlas-badges:family-crate-badge:start -->
[![{{ crate_name }}](https://img.shields.io/crates/v/{{ crate_name }}?label={{ crate_badge_label }}&logo=rust)]({{ crate_registry_url }})
<!-- bijux-atlas-badges:family-crate-badge:end -->

## Runtime Docs Badge

<!-- bijux-atlas-badges:family-rust-docs-badge:start -->
[![{{ crate_name }} rust-docs](https://img.shields.io/badge/rust--docs-{{ crate_badge_label }}-DEA584?logo=rust&logoColor=white)]({{ crate_docs_url }})
<!-- bijux-atlas-badges:family-rust-docs-badge:end -->

## Handbook Docs Badge

<!-- bijux-atlas-badges:family-docs-badge:start -->
[![{{ docs_badge_alt }}](https://img.shields.io/badge/docs-{{ docs_badge_label }}-2563EB?logo=materialformkdocs&logoColor=white)]({{ docs_url }})
<!-- bijux-atlas-badges:family-docs-badge:end -->
