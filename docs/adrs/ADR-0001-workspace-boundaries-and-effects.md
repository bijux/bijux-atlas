# ADR-0001: Workspace Boundaries And Effects

Status: Accepted

Context:
Atlas must keep deterministic core/model/query logic isolated from effectful runtime components.

Decision:
- Enforce one-way crate boundaries.
- Restrict I/O effects to store/server wiring paths.
- Guard boundaries with tests and policy scripts.

Consequences:
- Better reviewability and lower coupling.
- More adapter code at crate seams.
