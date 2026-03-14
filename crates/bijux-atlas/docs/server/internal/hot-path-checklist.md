# Hot Path Checklist

- Do not clone large response buffers on `/v1/genes` code paths.
- Prefer pre-allocated serialization buffers (`Vec::with_capacity` + `to_writer`).
- Keep heavy query execution behind bounded bulkheads (`class_heavy`, `heavy_workers`).
- Maintain strict response size guard before writing to socket.
- Keep per-dataset query concurrency bounded via cache-manager semaphores.
- Run clippy with `clippy::redundant_clone` denied in hot-path modules.
