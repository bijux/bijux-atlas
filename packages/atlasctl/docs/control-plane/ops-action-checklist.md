# How To Add a New Ops Action

Use this checklist when adding a new `atlasctl ops ...` action.

1. Add the action to the correct area command/runtime module (`commands/ops/<area>/command.py` + `runtime.py`).
2. Add/update the action inventory surface (`atlasctl ops --list-actions`) and verify it appears in `docs/_generated/ops-actions.md`.
3. Declare command capabilities in `configs/ops/command-capabilities.json`:
   - tools required
   - network requirement
   - supported profiles
4. Add schema/report contract(s) if the action emits JSON output.
5. Ensure writes are deterministic and under allowed roots (`artifacts/evidence/<area>/<run_id>/...`).
6. Add/attach the action to at least one ops suite (`configs/ops/suites.json`).
7. Add tests for:
   - invocation rendering / dry-run behavior
   - schema-valid success/failure payloads (if applicable)
8. Run boundary checks and suite coverage checks before merging.
