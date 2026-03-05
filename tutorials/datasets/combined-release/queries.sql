-- named_query: total_record_count
-- query_class: correctness
SELECT COUNT(*) AS total_record_count FROM combined_release;

-- named_query: sample_lookup_latency
-- query_class: performance
SELECT * FROM combined_release ORDER BY id LIMIT 10;
