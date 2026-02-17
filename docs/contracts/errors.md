# Error Codes Contract

- Owner: `docs-governance`

## What

Defines the `Error Codes Contract` registry contract.

## Why

Prevents drift between SSOT JSON, generated code, and operational consumers.

## Scope

Applies to producers and consumers of this registry.

## Non-goals

Does not define implementation internals outside this contract surface.

## Contracts

- `Internal`: stable machine error code for API and CLI contract surfaces.
- `InvalidCursor`: stable machine error code for API and CLI contract surfaces.
- `InvalidQueryParameter`: stable machine error code for API and CLI contract surfaces.
- `MissingDatasetDimension`: stable machine error code for API and CLI contract surfaces.
- `NotReady`: stable machine error code for API and CLI contract surfaces.
- `PayloadTooLarge`: stable machine error code for API and CLI contract surfaces.
- `QueryRejectedByPolicy`: stable machine error code for API and CLI contract surfaces.
- `RateLimited`: stable machine error code for API and CLI contract surfaces.
- `ResponseTooLarge`: stable machine error code for API and CLI contract surfaces.
- `Timeout`: stable machine error code for API and CLI contract surfaces.

## Failure modes

Invalid or drifted registry content is rejected by contract checks and CI gates.

## Examples

### `Internal`
```json
{
  "error": {
    "code": "Internal",
    "details": {
      "field": "example"
    },
    "message": "Internal error"
  }
}
```

### `InvalidCursor`
```json
{
  "error": {
    "code": "InvalidCursor",
    "details": {
      "field": "example"
    },
    "message": "InvalidCursor error"
  }
}
```

### `InvalidQueryParameter`
```json
{
  "error": {
    "code": "InvalidQueryParameter",
    "details": {
      "field": "example"
    },
    "message": "InvalidQueryParameter error"
  }
}
```

### `MissingDatasetDimension`
```json
{
  "error": {
    "code": "MissingDatasetDimension",
    "details": {
      "field": "example"
    },
    "message": "MissingDatasetDimension error"
  }
}
```

### `NotReady`
```json
{
  "error": {
    "code": "NotReady",
    "details": {
      "field": "example"
    },
    "message": "NotReady error"
  }
}
```

### `PayloadTooLarge`
```json
{
  "error": {
    "code": "PayloadTooLarge",
    "details": {
      "field": "example"
    },
    "message": "PayloadTooLarge error"
  }
}
```

### `QueryRejectedByPolicy`
```json
{
  "error": {
    "code": "QueryRejectedByPolicy",
    "details": {
      "field": "example"
    },
    "message": "QueryRejectedByPolicy error"
  }
}
```

### `RateLimited`
```json
{
  "error": {
    "code": "RateLimited",
    "details": {
      "field": "example"
    },
    "message": "RateLimited error"
  }
}
```

### `ResponseTooLarge`
```json
{
  "error": {
    "code": "ResponseTooLarge",
    "details": {
      "field": "example"
    },
    "message": "ResponseTooLarge error"
  }
}
```

### `Timeout`
```json
{
  "error": {
    "code": "Timeout",
    "details": {
      "field": "example"
    },
    "message": "Timeout error"
  }
}
```

## How to verify

```bash
$ make ssot-check
$ make docs-freeze
```

Expected output: both commands exit status 0 and print contract generation/check success.

## See also

- [Contracts Index](contracts-index.md)
- [SSOT Workflow](ssot-workflow.md)
- [Terms Glossary](../_style/terms-glossary.md)
