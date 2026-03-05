-- named_query: total_record_count
-- query_class: correctness
SELECT COUNT(*) AS total_record_count FROM genes_baseline;

-- named_query: sample_lookup_latency
-- query_class: performance
SELECT * FROM genes_baseline ORDER BY id LIMIT 10;
