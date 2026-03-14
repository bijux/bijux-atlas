-- named_query: total_record_count
-- query_class: correctness
SELECT COUNT(*) AS total_record_count FROM population_medium;

-- named_query: sample_lookup_latency
-- query_class: performance
SELECT * FROM population_medium ORDER BY id LIMIT 10;
