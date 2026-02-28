# Storage

Owner: `architecture`  
Type: `concept`  
Reason to exist: describe serving-store structure, index behavior, and cache interaction.

## Storage Model

- Serving store is release-indexed and immutable once published.
- Query execution relies on stable schemas and indexes.
- Dataset cache layers are bounded by policy-controlled limits.

## Reliability Rules

- Checksum mismatch blocks dataset open.
- Missing required indexes fail validation checks.
- Cache miss behavior degrades predictably without mutating source artifacts.

## Operational Relevance

Storage invariants define readiness behavior and prevent silent data drift during deployments.
