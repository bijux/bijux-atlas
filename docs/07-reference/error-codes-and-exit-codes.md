---
title: Error Codes and Exit Codes
audience: mixed
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Error Codes and Exit Codes

Atlas uses structured error reporting to keep failures classifiable and automatable.

## Error Reporting Model

```mermaid
flowchart LR
    Failure[Failure] --> ApiError[API error code]
    Failure --> ExitCode[CLI exit behavior]
    ApiError --> Client[Client handling]
    ExitCode --> Automation[Automation handling]
```

## Main API Error Categories

```mermaid
flowchart TD
    Errors[API error categories] --> Query[Query and validation]
    Errors --> Auth[Authentication and access]
    Errors --> Data[Dataset and artifact]
    Errors --> Runtime[Runtime and readiness]
```

## Common API Error Classes

- authentication and access errors
- invalid query or missing dataset dimension errors
- dataset or gene not found errors
- policy rejection and cost rejection errors
- readiness, timeout, and upstream availability errors
- artifact corruption or quarantine errors

## Important Point

Use the structured error code, not only the human message text, when building client or automation behavior.

