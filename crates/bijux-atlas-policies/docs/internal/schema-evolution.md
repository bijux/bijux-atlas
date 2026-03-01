# Evolving Policy Schema Safely

Procedure:
1. Add fields as additive, required-with-compatibility plan.
2. Update `policy.schema.json` required and properties in same change.
3. Add table-driven validation tests for new fields.
4. Update `policy.json` with explicit values (no silent defaults).
5. If version changes, apply monotonic bump and compatibility test updates.
