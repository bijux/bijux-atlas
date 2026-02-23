# Check Effects Vocabulary

Checks declare side-effect capabilities explicitly.

## Effects

- `fs_read`: read files in repository scope
- `fs_write`: write files (restricted to approved roots)
- `subprocess`: execute child processes
- `git`: invoke git state inspection as an explicit capability
- `network`: remote network access

## Default-Deny Policy

- default allowed effect: `fs_read`
- all other effects require explicit declaration in check metadata
- runner capabilities must allow declared effects, otherwise check is skipped with structured reason

## Write Discipline

- write-enabled checks must use approved roots
- evidence/report outputs are restricted to `artifacts/evidence/<run-id>/...`
- out-of-policy paths are hard failures
