-- named_query: total_record_count
-- query_class: correctness
SELECT COUNT(*) AS total_record_count FROM clinvar_large_sample;

-- named_query: sample_lookup_latency
-- query_class: performance
SELECT * FROM clinvar_large_sample ORDER BY id LIMIT 10;
