# Metrics Philosophy

Atlas metrics must be actionable, bounded, and durable.

Principles:

- Every metric must map to an operator decision.
- Labels must stay cardinality-safe under production load.
- Metric names and units must be explicit and stable.
- Breaking changes require versioned migration.
- Experimental metrics must be clearly marked and isolated from SLO decisions.
