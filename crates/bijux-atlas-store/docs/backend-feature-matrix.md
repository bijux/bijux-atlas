# Backend Feature Matrix

| Feature Set | Enabled Cargo Features | Backend Types | Network Dependency Surface |
| --- | --- | --- | --- |
| Minimal local runtime | `backend-local` | `LocalFsStore` | none |
| Remote object integration | `backend-s3` | `LocalFsStore`, `HttpReadonlyStore`, `S3LikeStore` | `reqwest` |

## Build Guidance

- Default builds use `backend-local` only.
- Enable remote backends explicitly with `--features backend-s3`.
- Use `validate_backend_compiled` at runtime selection boundaries to reject unavailable backends with clear messages.
