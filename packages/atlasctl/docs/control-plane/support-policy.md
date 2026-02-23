# Ops/Product Support Policy

This repository treats `ops` and `product` command surfaces as supported control-plane interfaces.

## Supported surfaces

- `atlasctl ops ...`
- `atlasctl product ...`

## Support expectations

- documented command surface
- deterministic reports/artifacts
- schema-validated outputs where applicable
- migration path for deprecated wrappers/scripts

## Non-goals

- direct shell script entrypoints under `ops/` as public interfaces
- undocumented make recipe behavior
