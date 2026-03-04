# Backward Compatibility Guarantee

Backward compatibility is the default for APIs, manifests, and operator workflows.
Breaking changes require explicit governance approval before merge and before release tagging.
Each approved break must include a migration guide, affected surface list, and rollback path.
Release notes must identify the first affected version and the last compatible version.
Validation coverage must include contract checks that fail when compatibility commitments drift.
