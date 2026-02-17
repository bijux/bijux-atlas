# Store Outage Behavior (Snippet)

During store outage:
- Prefer cached-only operation path when configured.
- Return stable error codes for network/cached-only conditions.
- Follow full operational playbook in `docs/runbooks/store-outage.md`.
