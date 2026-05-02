# Dev Atlas Runtime Ownership Boundary

`bijux-dev-atlas` is a maintainer control plane. It must not own runtime product behavior.

Forbidden ownership in this crate:

- ingest normalization and source parsing semantics
- dataset query planning and execution semantics
- server route behavior and HTTP runtime policy decisions
- end-user CLI behavior for `bijux-atlas` runtime commands

Allowed ownership in this crate:

- repository governance validation and policy checks
- documentation, release, and operations control-plane workflows
- evidence and report generation for maintainer use
