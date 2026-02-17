# Bijux Deprecation Policy

- Minimum deprecation window: one minor release cycle.
- Deprecation requires:
  - changelog entry
  - replacement guidance
  - removal target version/date
- Machine-visible APIs should return stable warning codes before removal.
- Breaking removal is allowed only after deprecation window expires.
