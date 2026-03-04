# Runtime Security Monitoring

Monitor authentication, authorization, integrity, and tamper signals continuously.

## Alert priorities

- Critical: sustained auth failure spikes, repeated authorization denials on admin routes, integrity/tamper violations.
- High: abrupt growth in invalid token signatures, revoked token reuse attempts.
- Medium: increased malformed request volume without policy failures.

## Immediate actions

1. Capture metrics, traces, and audit logs for the same time window.
2. Confirm scope by route, principal class, and auth mode.
3. Apply containment (temporary block rules, key rotation, or traffic shedding).
4. Record event timeline and remediation evidence in security incident artifacts.
