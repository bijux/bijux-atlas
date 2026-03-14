# Adding New Filters Safely

Checklist:
1. Add index strategy first.
2. Add filter parse/validation and limits checks.
3. Add SQL planner logic with deterministic ordering.
4. Add EXPLAIN PLAN tests proving indexed path.
5. Add cost-estimator adjustment and tests.
6. Update pagination/query-hash normalization if filter changes request shape.
