# ADR-0003: Federated Registry With Deterministic Merge

Status: Accepted

Context:
Production deployments may need multiple artifact registries with fallback.

Decision:
- Support multiple registry sources with explicit priority.
- Deterministically merge catalogs (first source wins).
- Track shadowing and expose registry health.

Consequences:
- Improved resilience and compatibility flexibility.
- Additional operational complexity around signatures and TTL/freeze modes.
