# Access To Audit Logs

Audit logs are operator-only operational records.

## Access model

- Operators may read audit logs for incident response, compliance review, and reliability analysis.
- Application callers do not receive raw audit log access.
- When file sink is enabled, the mounted audit path should remain accessible only to the workload
  operator boundary.
