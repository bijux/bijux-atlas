# Attack Surface Budget

`bijux-atlas-store` constrains remote dependency exposure by default.

## Budget Rules

- Default feature set must not pull network clients.
- Remote HTTP/S3 code paths must compile only when `backend-s3` is enabled.
- Local-only release profiles should compile with `--no-default-features --features backend-local`.

## Verification

- CI checks both `backend-local` and `backend-s3` feature builds.
- CI publishes dependency trees for both feature sets as artifacts.
- Runtime backend selection must fail fast if a requested backend was not compiled in.
