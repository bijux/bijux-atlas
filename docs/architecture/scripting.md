---
title: Scripting Architecture
audience: contributor
type: architecture
stability: stable
owner: dev-atlas
last_reviewed: 2026-03-03
tags:
  - automation
  - control-plane
---

# Scripting Architecture

Repository automation entrypoints are Rust-native and routed through `bijux dev atlas ...`.

Runtime product CLI commands are routed through `bijux atlas ...`.

Python tooling documents are historical-only and do not define active repository automation entrypoints.
