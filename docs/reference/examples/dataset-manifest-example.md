# Dataset Manifest Example

- Owner: `bijux-atlas-store`
- Audience: `user`
- Type: `reference`
- Stability: `stable`
- Reason to exist: provide a concrete example of dataset manifest fields used during validation and promotion.

## Example

```json
{
  "dataset_id": {
    "release": "110",
    "species": "homo_sapiens",
    "assembly": "GRCh38"
  },
  "artifact_digest": "sha256:1111111111111111111111111111111111111111111111111111111111111111",
  "schema_version": "v1",
  "record_count": 2,
  "generated_at": "2026-03-04T00:00:00Z"
}
```

## Related

- [Examples Index](index.md)
- [Dataset Operations](../dataset-operations.md)
- [Artifact Contracts](../contracts/artifacts/index.md)
