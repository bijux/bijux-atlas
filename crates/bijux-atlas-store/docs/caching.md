# Caching Semantics

HTTP/S3-like backends:
- Optional local cache by object key.
- `cached_only_mode=true` means no network fallback.
- HTTP backend supports ETag/If-None-Match with cached 304 handling.

Catalog/artifact reads should be deterministic with cache hit preference when configured.
