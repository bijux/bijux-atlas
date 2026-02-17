# Store Docs Index

Responsibilities:
- Enforce artifact publish/read contract at storage boundary.
- Provide local and remote backends with stable error mapping.
- Keep atomic publish and immutability guarantees explicit.

Strict boundaries:
- Store must not depend on API/server frameworks.
- Store owns storage effects only (filesystem/network), not query execution.

Docs:
- [Architecture](ARCHITECTURE.md)
- [Public API](PUBLIC_API.md)
- [Artifact contract](ARTIFACT_CONTRACT.md)
- [Effects policy](EFFECTS.md)
- [Caching semantics](CACHING.md)
- [Failure modes](FAILURE_MODES.md)
- [Rollback workflow](ROLLBACK.md)
- [Store outage runbook snippet](RUNBOOK_SNIPPET.md)
