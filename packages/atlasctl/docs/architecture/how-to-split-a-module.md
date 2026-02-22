# How To Split A Module

Use this process when a module or package approaches budget limits.

1. Identify the pressure
- Run `atlasctl policies culprits largest-files --report json`.
- Run `atlasctl policies culprits files-per-dir --report json`.
- Run `atlasctl policies culprits modules-per-dir --report json`.
- Run `atlasctl dev split-module <path>` to generate a seam-first split plan.

2. Choose a stable seam
- Split by responsibility, not by arbitrary line count.
- Keep I/O boundaries explicit in one module and move pure logic into focused helpers.

3. Preserve CLI/runtime contracts
- Keep entrypoint behavior and output schema unchanged.
- If paths are part of invoked workflows, update all references in makefiles/docs/tests.

4. Keep intent visible
- Every new package directory needs `README.md` or a clearly named check module.
- Avoid placeholder names (`part1`, `chunk`, `tmp`).

5. Verify and gate
- Run affected tests.
- Run `atlasctl check repo --format json`.
- Run `atlasctl suite run fast`.

6. Commit in small batches
- First commit: mechanical moves/renames.
- Second commit: behavioral refactor.
- Third commit: policy/docs/tests.
