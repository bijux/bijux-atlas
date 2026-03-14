# Runtime Startup Config

Source of truth for startup config resolution used by `atlas-server`.

Resolution precedence: `CLI > ENV > config file > defaults`.

| Field | CLI Flag | ENV | Config Key | Default |
|---|---|---|---|---|
| `bind_addr` | `--bind` | `ATLAS_BIND` | `bind_addr` | `0.0.0.0:8080` |
| `store_root` | `--store-root` | `ATLAS_STORE_ROOT` | `store_root` | `artifacts/server-store` |
| `cache_root` | `--cache-root` | `ATLAS_CACHE_ROOT` | `cache_root` | `artifacts/server-cache` |

File formats: `.json`, `.yaml`/`.yml`, `.toml`.
Validation: all resolved fields are required and must be non-empty.
