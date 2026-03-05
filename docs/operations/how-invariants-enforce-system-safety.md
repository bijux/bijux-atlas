# How Invariants Enforce System Safety

Invariant checks form a hard safety gate for Atlas operational correctness.

Safety model:

1. Invariants validate config, registry, and profile relationships.
2. If any invariant fails, runtime start gating invariant fails.
3. Failing invariants produce deterministic reports and stable exit codes.

This allows operators and CI pipelines to block unsafe starts and catch drift before serving traffic.
