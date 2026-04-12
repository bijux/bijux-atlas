---
title: Compatibility Review Checklist
audience: maintainers
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-12
---

# Compatibility Review Checklist

Use this checklist when a change might alter a repository-owned Atlas promise.

## Review Questions

- does the change affect a documented API, config, output, or artifact rule
- is the behavior covered by compatibility or contract tests
- does release evidence need to call out the change explicitly
- are redirects, docs, and generated references still aligned

## Outcome

The goal is to keep contract changes intentional, reviewable, and visible
before they escape as accidental drift.
