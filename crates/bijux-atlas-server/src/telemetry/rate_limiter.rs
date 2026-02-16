use crate::telemetry::redis_backend::RedisBackend;
use crate::*;

#[derive(Debug, Clone)]
struct Bucket {
    tokens: f64,
    last_refill: Instant,
}

#[derive(Default)]
pub(crate) struct RateLimiter {
    buckets: Mutex<HashMap<String, Bucket>>,
    redis: Option<RedisBackend>,
    scope: String,
}

impl RateLimiter {
    pub(crate) fn new(redis: Option<RedisBackend>, scope: &str) -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
            redis,
            scope: scope.to_string(),
        }
    }

    pub(crate) async fn allow(&self, key: &str, cfg: &RateLimitConfig) -> bool {
        if let Some(redis) = &self.redis {
            match redis.rate_limit_allow(&self.scope, key, cfg).await {
                Ok(v) => return v,
                Err(e) => {
                    tracing::warn!(scope = %self.scope, "redis rate-limit fallback: {e}");
                }
            }
        }
        let now = Instant::now();
        let mut lock = self.buckets.lock().await;
        let bucket = lock.entry(key.to_string()).or_insert_with(|| Bucket {
            tokens: cfg.capacity,
            last_refill: now,
        });
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.last_refill = now;
        bucket.tokens = (bucket.tokens + (elapsed * cfg.refill_per_sec)).min(cfg.capacity);
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}
