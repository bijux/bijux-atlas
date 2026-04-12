---
title: Logging Contracts
audience: operators
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-13
---

# Logging Contracts

Logging format and coverage are governed explicitly so downstream diagnostics
can rely on stable fields and evidence quality.

## Purpose

Use this page to understand which log fields are required, how events are
classified, and which sensitive content must never appear in Atlas logs.

## Source of Truth

- `ops/observe/logging/format-validator-contract.json`
- `ops/observe/contracts/logs-fields-contract.json`
- `ops/observe/contracts/log-classification-contract.json`
- `ops/observe/contracts/logs-sample.jsonl`

## Required Fields and Stability Rules

`ops/observe/contracts/logs-fields-contract.json` currently requires
`level`, `msg`, and `request_id`, and also declares `event_name` as part of the
required fields registry. The contract keeps allowed log levels stable and ties
specific event names to their required fields.

## Classification Rules

`ops/observe/contracts/log-classification-contract.json` classifies events by
prefix into runtime, query, ingest, artifact, configuration, startup, shutdown,
and security classes. Unknown events are violations unless they are explicitly
registered.

## Prohibited Sensitive Content

The log field contract explicitly prohibits sensitive fields such as:

- `email`
- `phone`
- `ip`
- `ssn`
- `name`

Operators should treat those fields as invalid output, not as content to clean
up after the fact.

## Validation Path

Validate logging changes against:

- the format validator contract
- the log field and event registry contract
- the classification contract
- the sample fixtures used to check machine-readable output shape

## Related Contracts and Assets

- `ops/observe/logging/format-validator-contract.json`
- `ops/observe/contracts/logs-fields-contract.json`
- `ops/observe/contracts/log-classification-contract.json`
- `ops/observe/contracts/logs-sample.jsonl`
