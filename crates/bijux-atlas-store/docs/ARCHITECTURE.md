# Architecture

## Architecture

Modules:
- `paths`: deterministic artifact and lock paths.
- `manifest`: lock/checksum verification primitives.
- `catalog`: strict catalog validation + canonicalization.
- `backend`: trait and backend implementations.

Backends:
- `LocalFsStore`
- `HttpReadonlyStore`
- `S3LikeStore`
