# Effects Policy

Allowed effects:
- Filesystem read/write (local backend).
- Network I/O (HTTP/S3-like backends).

Disallowed effects:
- Server runtime concerns (routing/middleware).
- Query planning/execution.

All storage effects must be explicit in backend methods.
