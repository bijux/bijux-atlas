# Ops Refactor Playbook

Use this playbook when refactoring `atlasctl` ops modules.

1. Identify the public surface (`command.py`) and runtime owner (`runtime.py` or `runtime_modules/*`).
2. Move business logic out of `command.py` first; keep `command.py` as thin dispatch.
3. Replace direct shell/script calls with:
   - `atlasctl.commands.ops.tools`
   - `atlasctl.core.process`
   - area runtime helpers
4. Preserve behavior with a compatibility shim only when necessary, and record the shim in the temporary shim approval file.
5. Add/adjust boundary checks instead of relying on code review memory:
   - import rules
   - no stdout writes in command entrypoints
   - no cwd reliance / no `Path('.')`
6. Update capability manifest (`configs/ops/command-capabilities.json`) if tool/network requirements change.
7. Update suites/docs/goldens if the action surface changes.
8. Delete the compatibility shim after parity is proven and remove allowlist entries in the same or next commit.
