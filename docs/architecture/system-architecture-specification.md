---
title: System Architecture Specification
audience: contributor
type: concept
stability: stable
owner: architecture
last_reviewed: 2026-03-04
tags:
  - architecture
  - system-model
related:
  - docs/architecture/index.md
  - docs/architecture/summary.md
  - docs/architecture/diagrams/index.md
---

# System Architecture Specification

This page is the canonical architecture specification for Atlas.
It defines the system model used by contributors, operators, and reviewers.

## System Boundary

Atlas owns deterministic ingest, validation, artifact production, serving, and operational evidence generation.
Atlas does not own third-party infrastructure lifecycle or external system policy.

## Data Plane

The data plane is responsible for ingesting, validating, storing, and querying datasets and artifacts.
It includes ingest processing, storage persistence, query execution, and API serving paths.

## Control Plane

The control plane is responsible for policy checks, contract enforcement, ops orchestration, release assembly, and evidence reporting.
It includes check runners, contract runners, docs and config validators, and release tooling.

## Subsystem Definitions

### Ingest Subsystem

Validates source inputs, enforces schema and policy, and produces deterministic dataset artifacts.

### Query Subsystem

Plans and executes query operations against approved artifacts with deterministic behavior and stable error models.

### Storage Subsystem

Stores datasets, indices, manifests, and related artifacts with integrity and reproducibility guarantees.

### API Subsystem

Exposes runtime capabilities through stable endpoints and contract-governed request and response behavior.

### CLI Subsystem

Provides operator and contributor entry points for control-plane operations, validation, and release workflows.

### Policy Subsystem

Defines, validates, and enforces checks and contracts across docs, configs, ops, release, and runtime boundaries.

## Runtime Lifecycle

1. Bootstrap runtime configuration from validated sources.
2. Load approved artifacts and manifests.
3. Start services with readiness gating.
4. Serve runtime operations under contract constraints.
5. Emit observability and evidence outputs.
6. Shutdown with deterministic cleanup behavior.

## Dataset Lifecycle

1. Source collection.
2. Dataset preparation.
3. Schema and policy validation.
4. Versioned artifact generation.
5. Promotion to approved usage.
6. Retention, archival, or removal.

## Artifact Lifecycle

1. Artifact creation from validated input.
2. Manifest attachment and digest computation.
3. Contract and policy verification.
4. Publication into artifact storage.
5. Consumption by runtime and operations.
6. Evidence retention and lineage reporting.

## Query Request Lifecycle

1. Request receipt and input parsing.
2. Parameter and authorization validation.
3. Query planning.
4. Execution against current artifact state.
5. Response serialization with stable schema.
6. Telemetry emission and completion accounting.

## Ingest Pipeline Stages

1. Input discovery and normalization.
2. Structural schema validation.
3. Semantic validation and policy checks.
4. Transformation and enrichment.
5. Deterministic ordering and canonicalization.
6. Artifact assembly and manifest generation.

## Query Planning Stages

1. Parse request and derive intent.
2. Validate allowed operations and filters.
3. Select dataset versions and index access paths.
4. Build execution plan.
5. Apply projection, pagination, and ordering strategy.
6. Execute and collect results deterministically.

## Shard Architecture

Atlas organizes large datasets into deterministic shard units with explicit shard metadata, reproducible shard boundaries, and stable lookup behavior.

## Dataset Versioning Model

Datasets are versioned with monotonic identifiers, explicit compatibility expectations, and immutable published outputs.

## Artifact Storage Model

Artifacts are stored in deterministic paths with digest-backed identity, manifest linkage, and evidence-compatible layout.

## Manifest Structure

Manifests include dataset identity, version metadata, digest references, schema and policy status, and provenance fields required by release and audit surfaces.
