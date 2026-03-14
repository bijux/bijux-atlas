# Rollback Workflow

Store-level rollback model:
- Do not mutate published dataset artifacts.
- Rollback by unpublishing/removing catalog entry pointing to the bad dataset.
- Keep immutable artifacts for audit/recovery unless retention policy requires cleanup.
