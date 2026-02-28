# User Stories

Owner: `product`  
Type: `concept`  
Reason to exist: map real user goals to stable Atlas query capabilities.

## Core Stories

1. As a researcher, I can find genes in an explicit dataset identity.
2. As a curator, I can inspect transcript details for a target gene.
3. As an operator, I can compare release-to-release gene differences.

## Example Queries

- `GET /v1/genes?release=112&species=homo_sapiens&assembly=GRCh38&name_prefix=BRCA&limit=5`
- `GET /v1/genes/ENSG00000139618/transcripts?release=112&species=homo_sapiens&assembly=GRCh38&limit=10`
- `GET /v1/diff/genes?from_release=111&to_release=112&species=homo_sapiens&assembly=GRCh38&limit=20`

## Related Pages

- [What Is Bijux Atlas](what-is-bijux-atlas.md)
- [Compatibility Promise](compatibility-promise.md)
- [API Surface](../api/v1-surface.md)
