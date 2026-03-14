# Example maturity policy

- `basic`: shortest path to run a useful query, minimal dependencies.
- `advanced`: production-adjacent workflows and larger data flow patterns.
- `integrations`: ecosystem-specific adapters (for example pandas or Airflow).

Promotion criteria from `basic` to `advanced`:
- explains a real operational pattern,
- remains deterministic with documented environment variables,
- stays concise and avoids framework sprawl.
