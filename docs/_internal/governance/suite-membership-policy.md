---
title: Suite Membership Policy
audience: maintainer
type: policy
status: canonical
owner: atlas-governance
last_reviewed: 2026-03-15
---

# Suite Membership Policy

## Membership boundary

Each governed entry should have one primary suite membership that matches its real purpose.
Checks stay in checks-oriented suites, and contracts stay in the `contracts` suite unless governance explicitly approves a different home.

## Allowed overlap

Overlap is only allowed when one governed entry must satisfy two distinct review surfaces and duplicating the implementation would reduce truthfulness.
When overlap exists, record the reason in registry metadata instead of hiding it in wrapper aliases.

## How to move an entry

Update the owning registry first, then update the suite registry, then update any make or workflow wrapper that exposes the entry.
Finish by rerunning the affected suite and registry validation so the surface and the evidence stay aligned.

## Purpose

This page documents internal Atlas repository behavior for maintainers and generated evidence around suite membership policy.

## Stability

This page is internal maintainer documentation. Keep it aligned with the current repository behavior, but do not treat it as a public runtime contract.
