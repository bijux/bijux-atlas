# Default Field Set (`/v1/genes`)

`include` controls optional fields for `GET /v1/genes`.

- Default response (no `include`): `gene_id`, `name`
- Optional include values:
  - `coords` -> `seqid`, `start`, `end`
  - `biotype` -> `biotype`
  - `counts` -> `transcript_count`
  - `length` -> `sequence_length`

Examples:

- Minimal:
  - `GET /v1/genes?release=110&species=homo_sapiens&assembly=GRCh38`
- Add coordinates and counts:
  - `GET /v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&include=coords,counts`

## Non-goal

`fields=` projection is not part of v1. Requests using `fields` are rejected with `400` and must use `include`.
