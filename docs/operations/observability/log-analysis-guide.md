# Log Analysis Guide

1. Group logs by `event_name` to isolate behavior classes.
2. Filter by `request_id` to reconstruct request timeline.
3. Join `trace_id` when available to map logs to spans.
4. Prioritize `WARN` and `ERROR` events with high recurrence.
5. Confirm redaction compliance before exporting evidence bundles.
