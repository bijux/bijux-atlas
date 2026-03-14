// SPDX-License-Identifier: Apache-2.0

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

    pub(crate) async fn allow_with_factor(
        &self,
        key: &str,
        cfg: &RateLimitConfig,
        factor: f64,
    ) -> bool {
        let factor = factor.clamp(0.1, 1.0);
        let effective = RateLimitConfig {
            capacity: (cfg.capacity * factor).max(1.0),
            refill_per_sec: (cfg.refill_per_sec * factor).max(0.5),
        };
        if let Some(redis) = &self.redis {
            match redis.rate_limit_allow(&self.scope, key, &effective).await {
                Ok(v) => return v,
                Err(e) => {
                    tracing::warn!(scope = %self.scope, "redis rate-limit fallback: {e}");
                }
            }
        }
        let now = Instant::now();
        let mut lock = self.buckets.lock().await;
        let bucket = lock.entry(key.to_string()).or_insert_with(|| Bucket {
            tokens: effective.capacity,
            last_refill: now,
        });
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.last_refill = now;
        bucket.tokens =
            (bucket.tokens + (elapsed * effective.refill_per_sec)).min(effective.capacity);
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}
