# Legacy Policy

Single policy for pre-`0.1` release:

- No legacy codepaths are allowed in `atlasctl`.
- `packages/atlasctl/src/atlasctl/legacy/` must be absent.
- Top-level commands `legacy`, `compat`, and `migration` must not exist.
- `atlasctl internal legacy inventory --report json` is the only allowed legacy audit surface.
