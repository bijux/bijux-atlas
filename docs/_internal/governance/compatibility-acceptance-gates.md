# Compatibility Acceptance Gates

- Owner: `bijux-atlas-governance`
- Type: `reference`
- Audience: `reviewers`
- Stability: `stable`

| Policy | Executable check |
| --- | --- |
| Compatibility policy file validates | `GOV-COMP-001` via `governance deprecations validate` |
| Deprecation registry validates | `GOV-DEP-001` via `governance deprecations validate` |
| Deprecations are not past removal target | `GOV-DEP-002` via `governance deprecations validate` |
| Prod profiles do not use deprecated keys | `OPS-COMP-001` via `governance breaking validate` |
| CI compatibility warnings are reported | `OPS-COMP-002` via `governance breaking validate` |
| Report schema migration notes exist | `GOV-REP-001` via `governance breaking validate` |
| Docs redirects cover moved pages | `DOCS-COMP-001` via `governance breaking validate` |
| Breaking changes report is generated | `GOV-BREAK-001` via `governance breaking validate` |
| Breaking changes have notes and version policy coverage | `GOV-BREAK-002` via `governance breaking validate` |
| Governance doctor report validates | `GOV-DOC-001` via `governance doctor` |
| Release evidence includes governance doctor report | `REL-GOV-001` via `governance doctor` |
| Release evidence includes institutional delta | `REL-GOV-002` via `governance doctor` |
