# Cookbook: Refactor A Check Without Breaking IDs

1. Keep `check_id` unchanged; move implementation only.
2. Preserve severity/tags unless policy intentionally changes.
3. Keep fix hints stable and actionable.
4. Move code first, keep temporary import bridge in the same PR only, then delete bridge.
5. Run suite and check inventory validation after refactor.
6. If behavior changes, update tests and document rationale in docs/decisions when needed.
