# Log Message Conventions

1. Messages are imperative and outcome-oriented, for example `request completed`.
2. Use stable `event_name` values; never overload one event for unrelated behavior.
3. Keep message text short and put structured context in fields.
4. Include `request_id` for request path events.
5. Include `trace_id` when trace context is available.
