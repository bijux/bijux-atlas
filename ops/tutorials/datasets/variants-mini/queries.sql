-- named_query: total_record_count
-- query_class: correctness
SELECT COUNT(*) AS total_record_count FROM variants_mini;

-- named_query: sample_lookup_latency
-- query_class: performance
SELECT * FROM variants_mini ORDER BY id LIMIT 10;
