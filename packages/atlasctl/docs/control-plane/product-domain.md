# Product Domain

`product` in atlas means the release-facing artifact set and the validation steps required before shipping.

## Scope

- Docker image build outputs and image tag intent
- Helm chart package outputs
- Product artifact inventory and checksums (`artifact-manifest.json`)
- Product validation and release publishing gates

## Canonical Flow

1. `atlasctl product build`
2. `atlasctl product validate`
3. `atlasctl product diff <old> <new>` (optional release comparison)
4. `atlasctl product publish --internal` (optional/internal publish step)

## Artifact Layout

- Build evidence: `artifacts/evidence/product/build/<run_id>/...`
- Artifact manifest: `artifacts/evidence/product/build/<run_id>/artifact-manifest.json`
- Chart packages: `artifacts/chart/*.tgz`

## Validation Expectations

- Manifest schema-valid against `configs/product/artifact-manifest.schema.json`
- Deterministic ordering of artifact rows
- Checksums match on-disk artifacts
- Product lanes emit reports under evidence roots only

