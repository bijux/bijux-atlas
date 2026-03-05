# Example Dataset Specification

This specification defines the example datasets shipped with Atlas documentation and tutorials.

## Goals

- Provide reproducible sample data for onboarding and integration testing.
- Keep schema shape consistent across minimal, medium, and large variants.
- Support ingest, query, filtering, aggregation, and streaming tutorial flows.

## Required Files

Each dataset directory must include:

- `genes.jsonl`: line-delimited JSON records.
- `metadata.json`: dataset identity, schema version, and record count.

## Data Contract

Required fields for each record:

- `gene_id` (string)
- `symbol` (string)
- `chromosome` (string)
- `biotype` (string)
- `length_bp` (integer)

Optional fields:

- `gc_content` (number)
- `description` (string)

## Dataset Tiers

- Minimal: 10-50 records for quick local validation.
- Medium: 1,000-10,000 records for realistic querying behavior.
- Large synthetic: 100,000+ records for load and pagination exercises.
