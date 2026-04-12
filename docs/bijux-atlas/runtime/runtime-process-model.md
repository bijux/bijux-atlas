---
title: Runtime Process Model
audience: mixed
type: concept
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-12
---

# Runtime Process Model

The Atlas runtime process is the composed application that binds config,
resolves stores, wires adapters, and exposes the HTTP surface.

## Process Responsibilities

- accept validated runtime configuration
- initialize store and cache dependencies
- expose health, readiness, metrics, and product endpoints
- keep request execution separate from build and maintainer control-plane work

## Reading Rule

When the question is about startup wiring or live process behavior, stay in the
runtime slice rather than the maintainer or operations handbooks.
