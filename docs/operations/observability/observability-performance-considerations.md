# Observability Performance Considerations

Performance guardrails:

- keep metric label cardinality bounded
- keep trace sampling ratio aligned with incident and cost profile
- avoid high-volume debug logging in normal operation
- use rotation and retention settings that fit storage budget

Validation:

- monitor request latency impact when enabling additional observability surfaces
- verify no exporter path introduces blocking behavior on critical request flow
