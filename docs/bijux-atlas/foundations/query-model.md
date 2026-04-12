---
title: Query Model
audience: mixed
type: concept
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-12
---

# Query Model

Atlas queries are validated requests over published dataset state.

That matters because query behavior is not just an endpoint shape. It is a
combination of dataset selection, request validation, cost control, and
structured response rules.

## Query Boundary

- clients ask for explicit dataset state
- policy and limits validate the request
- runtime executes against immutable published content
- responses follow documented output contracts

## Reading Rule

If a question is about request semantics, response shape, or API expectations,
it belongs to the repository handbook rather than the operations handbook.
