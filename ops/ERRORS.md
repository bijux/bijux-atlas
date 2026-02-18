# Ops Error Registry

SSOT source: `ops/_meta/error-registry.json` (`schema_version: v1`).

- `10` `OPS_ERR_CONFIG`: missing/invalid env or config contract.
- `11` `OPS_ERR_CONTEXT`: cluster/profile context guard failed.
- `12` `OPS_ERR_VERSION`: local tool versions drift from pinned versions.
- `13` `OPS_ERR_PREREQ`: required tool missing.
- `14` `OPS_ERR_TIMEOUT`: bounded operation exceeded timeout.
- `15` `OPS_ERR_VALIDATION`: schema/manifest/contract validation failure.
- `16` `OPS_ERR_ARTIFACT`: artifact policy/path violation.
- `17` `OPS_ERR_DOCS`: docs policy violation for supported entrypoints.
- `99` `OPS_ERR_INTERNAL`: unexpected internal failure.
