# Image Tags And Compatibility

- Supported tags are defined in `release/images-v0.1.toml`.
- `latest` is forbidden unless policy explicitly allows it.
- Prefer digest pinning in production rollouts.
- Image version binding is workspace version for `v0.1`.
- Compatibility contract: image major/minor tracks release compatibility promises.
