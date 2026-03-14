# Store Outage Behavior (Snippet)

During store outage:
- Prefer cached-only operation path when configured.
- Return stable error codes for network/cached-only conditions.
- Follow the full operational playbook in `../../../operations/runbooks/store-outage.md`.
