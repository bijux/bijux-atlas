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

- `ArtifactCorrupted`: stable machine error code for API and CLI contract surfaces.
- `ArtifactQuarantined`: stable machine error code for API and CLI contract surfaces.
- `DatasetNotFound`: stable machine error code for API and CLI contract surfaces.
- `GeneNotFound`: stable machine error code for API and CLI contract surfaces.
- `IngestDuplicateTranscriptId`: stable machine error code for API and CLI contract surfaces.
- `IngestInvalidCdsPhase`: stable machine error code for API and CLI contract surfaces.
- `IngestInvalidStrand`: stable machine error code for API and CLI contract surfaces.
- `IngestMissingParent`: stable machine error code for API and CLI contract surfaces.
- `IngestMissingRequiredField`: stable machine error code for API and CLI contract surfaces.
- `IngestMissingTranscriptId`: stable machine error code for API and CLI contract surfaces.
- `IngestMultiParentChild`: stable machine error code for API and CLI contract surfaces.
- `IngestMultiParentTranscript`: stable machine error code for API and CLI contract surfaces.
- `IngestSeqidCollision`: stable machine error code for API and CLI contract surfaces.
- `IngestUnknownFeature`: stable machine error code for API and CLI contract surfaces.
- `Internal`: stable machine error code for API and CLI contract surfaces.
- `InvalidCursor`: stable machine error code for API and CLI contract surfaces.
- `InvalidQueryParameter`: stable machine error code for API and CLI contract surfaces.
- `MissingDatasetDimension`: stable machine error code for API and CLI contract surfaces.
- `NotReady`: stable machine error code for API and CLI contract surfaces.
- `PayloadTooLarge`: stable machine error code for API and CLI contract surfaces.
- `QueryRejectedByPolicy`: stable machine error code for API and CLI contract surfaces.
- `QueryTooExpensive`: stable machine error code for API and CLI contract surfaces.
- `RangeTooLarge`: stable machine error code for API and CLI contract surfaces.
- `RateLimited`: stable machine error code for API and CLI contract surfaces.
- `ResponseTooLarge`: stable machine error code for API and CLI contract surfaces.
- `Timeout`: stable machine error code for API and CLI contract surfaces.
- `UpstreamStoreUnavailable`: stable machine error code for API and CLI contract surfaces.
- `ValidationFailed`: stable machine error code for API and CLI contract surfaces.

## Failure modes

Invalid or drifted registry content is rejected by contract checks and CI gates.

## Examples

### `ArtifactCorrupted`
```json
{
  "error": {
    "code": "ArtifactCorrupted",
    "details": {
      "field": "example"
    },
    "message": "ArtifactCorrupted error"
  }
}
```

### `ArtifactQuarantined`
```json
{
  "error": {
    "code": "ArtifactQuarantined",
    "details": {
      "field": "example"
    },
    "message": "ArtifactQuarantined error"
  }
}
```

### `DatasetNotFound`
```json
{
  "error": {
    "code": "DatasetNotFound",
    "details": {
      "field": "example"
    },
    "message": "DatasetNotFound error"
  }
}
```

### `GeneNotFound`
```json
{
  "error": {
    "code": "GeneNotFound",
    "details": {
      "field": "example"
    },
    "message": "GeneNotFound error"
  }
}
```

### `IngestDuplicateTranscriptId`
```json
{
  "error": {
    "code": "IngestDuplicateTranscriptId",
    "details": {
      "field": "example"
    },
    "message": "IngestDuplicateTranscriptId error"
  }
}
```

### `IngestInvalidCdsPhase`
```json
{
  "error": {
    "code": "IngestInvalidCdsPhase",
    "details": {
      "field": "example"
    },
    "message": "IngestInvalidCdsPhase error"
  }
}
```

### `IngestInvalidStrand`
```json
{
  "error": {
    "code": "IngestInvalidStrand",
    "details": {
      "field": "example"
    },
    "message": "IngestInvalidStrand error"
  }
}
```

### `IngestMissingParent`
```json
{
  "error": {
    "code": "IngestMissingParent",
    "details": {
      "field": "example"
    },
    "message": "IngestMissingParent error"
  }
}
```

### `IngestMissingRequiredField`
```json
{
  "error": {
    "code": "IngestMissingRequiredField",
    "details": {
      "field": "example"
    },
    "message": "IngestMissingRequiredField error"
  }
}
```

### `IngestMissingTranscriptId`
```json
{
  "error": {
    "code": "IngestMissingTranscriptId",
    "details": {
      "field": "example"
    },
    "message": "IngestMissingTranscriptId error"
  }
}
```

### `IngestMultiParentChild`
```json
{
  "error": {
    "code": "IngestMultiParentChild",
    "details": {
      "field": "example"
    },
    "message": "IngestMultiParentChild error"
  }
}
```

### `IngestMultiParentTranscript`
```json
{
  "error": {
    "code": "IngestMultiParentTranscript",
    "details": {
      "field": "example"
    },
    "message": "IngestMultiParentTranscript error"
  }
}
```

### `IngestSeqidCollision`
```json
{
  "error": {
    "code": "IngestSeqidCollision",
    "details": {
      "field": "example"
    },
    "message": "IngestSeqidCollision error"
  }
}
```

### `IngestUnknownFeature`
```json
{
  "error": {
    "code": "IngestUnknownFeature",
    "details": {
      "field": "example"
    },
    "message": "IngestUnknownFeature error"
  }
}
```

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

### `QueryTooExpensive`
```json
{
  "error": {
    "code": "QueryTooExpensive",
    "details": {
      "field": "example"
    },
    "message": "QueryTooExpensive error"
  }
}
```

### `RangeTooLarge`
```json
{
  "error": {
    "code": "RangeTooLarge",
    "details": {
      "field": "example"
    },
    "message": "RangeTooLarge error"
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

### `UpstreamStoreUnavailable`
```json
{
  "error": {
    "code": "UpstreamStoreUnavailable",
    "details": {
      "field": "example"
    },
    "message": "UpstreamStoreUnavailable error"
  }
}
```

### `ValidationFailed`
```json
{
  "error": {
    "code": "ValidationFailed",
    "details": {
      "field": "example"
    },
    "message": "ValidationFailed error"
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
