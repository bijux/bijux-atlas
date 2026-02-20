# Versioning Alignment

`atlasctl` version tracks repository versioning and commit lineage.

Current rule:
- CLI reports `0.1.0+<git_sha>` and is tied to the checked-out repo state.

Future rule (once public):
- patch releases must preserve backward compatibility for documented commands and JSON outputs.
