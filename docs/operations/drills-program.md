# Drills Program

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@c59da0bf`
- Reason to exist: define drill catalog, schedule, evidence, and ownership.

## Drill catalog

- Incident response drill: command, communication, and escalation timing.
- Store outage drill: mitigation and rollback under load.
- Dataset integrity drill: detection and recovery for corruption signals.
- Release rollback drill: failed rollout recovery to known good state.

## Schedule and ownership

- Weekly: incident response and store outage drills.
- Bi-weekly: dataset integrity drill.
- Monthly: release rollback drill.
- Drill lead: `bijux-atlas-operations` on-call primary.

## Evidence requirements

- Drill date, participants, and scenario ID.
- Time-to-detect and time-to-mitigate.
- Follow-up actions with owners and due dates.

## Verify success

A drill is successful when the team completes mitigation and publishes evidence within one business day.

## Next

- [Incident Response](incident-response.md)
- [Runbooks](runbooks/index.md)
