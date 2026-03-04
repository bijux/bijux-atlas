# Respond To Suspicious Activity

- Owner: `bijux-atlas-security`
- Type: `runbook`
- Audience: `operators`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define the first response workflow when audit logs indicate suspicious access.

## Prereqs

- Audit logging is enabled.
- Access to `artifacts/security/audit-verify.json` and the active audit sink.

## Verify

- Confirm the relevant event appears in the `atlas_audit` stream or retained audit file.
- Confirm the event fields match the governed inventory and do not include redacted secret markers.
- Confirm the matching evidence bundle still verifies cleanly if release artifacts are involved.

## Response

- Preserve the relevant audit window before log rotation removes it.
- Compare `principal`, `action`, and `resource_id` with the declared auth model and policy.
- Check whether the event came from an allowed administrative action or an unexpected route.
- Escalate through the institutional review path if the action crosses the declared trust boundary.

## Rollback

- Revert temporary containment changes once the event is classified and documented.
- Remove any emergency overrides only after the related evidence and reviewer notes are attached.
