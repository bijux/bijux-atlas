# Add Governance Checks And Contracts

## Add a check

1. Define the check objective and stable identifier.
2. Implement a deterministic evaluation path.
3. Add fixtures for pass/fail examples.
4. Add CLI invocation coverage.
5. Add docs and evidence path expectations.

## Add a contract

1. Define explicit invariant and failure text.
2. Keep the scope narrow and machine-verifiable.
3. Ensure output is deterministic in text and JSON.
4. Add contract execution coverage in CI and local workflows.

## Required quality bars

1. Exact file-path reporting for violations.
2. No hidden mutable state.
3. Reproducible output for the same repository state.
