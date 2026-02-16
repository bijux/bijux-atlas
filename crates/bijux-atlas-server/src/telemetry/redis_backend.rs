use crate::RateLimitConfig;
use redis::AsyncCommands;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, OwnedMutexGuard};
use tokio::time::timeout;

#[derive(Clone, Debug)]
pub(crate) struct RedisPolicy {
    pub timeout: Duration,
    pub retry_attempts: usize,
    pub breaker_failure_threshold: u32,
    pub breaker_open_duration: Duration,
    pub max_key_bytes: usize,
    pub max_cardinality: usize,
    pub max_ttl_secs: usize,
}

impl Default for RedisPolicy {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(50),
            retry_attempts: 2,
            breaker_failure_threshold: 8,
            breaker_open_duration: Duration::from_millis(3000),
            max_key_bytes: 256,
            max_cardinality: 100_000,
            max_ttl_secs: 60,
        }
    }
}

#[derive(Default)]
struct RedisBreakerState {
    failure_count: u32,
    open_until: Option<Instant>,
}

#[derive(Default)]
pub(crate) struct RedisMetrics {
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    pub read_fallbacks: AtomicU64,
    pub write_fallbacks: AtomicU64,
    pub rate_limit_fallbacks: AtomicU64,
    pub breaker_open_total: AtomicU64,
    pub breaker_reject_total: AtomicU64,
    pub key_reject_total: AtomicU64,
    pub cardinality_reject_total: AtomicU64,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RedisMetricsSnapshot {
    pub hits: u64,
    pub misses: u64,
    pub read_fallbacks: u64,
    pub write_fallbacks: u64,
    pub rate_limit_fallbacks: u64,
    pub breaker_open_total: u64,
    pub breaker_reject_total: u64,
    pub key_reject_total: u64,
    pub cardinality_reject_total: u64,
    pub tracked_keys: u64,
}

#[derive(Clone)]
pub(crate) struct RedisBackend {
    client: redis::Client,
    prefix: String,
    policy: RedisPolicy,
    breaker: Arc<Mutex<RedisBreakerState>>,
    inflight: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
    key_registry: Arc<Mutex<HashSet<String>>>,
    pub metrics: Arc<RedisMetrics>,
}

impl RedisBackend {
    pub(crate) fn new(url: &str, prefix: &str, policy: RedisPolicy) -> Result<Self, String> {
        let client = redis::Client::open(url).map_err(|e| e.to_string())?;
        Ok(Self {
            client,
            prefix: prefix.to_string(),
            policy,
            breaker: Arc::new(Mutex::new(RedisBreakerState::default())),
            inflight: Arc::new(Mutex::new(HashMap::new())),
            key_registry: Arc::new(Mutex::new(HashSet::new())),
            metrics: Arc::new(RedisMetrics::default()),
        })
    }

    async fn breaker_check(&self) -> Result<(), String> {
        let lock = self.breaker.lock().await;
        if let Some(until) = lock.open_until {
            if Instant::now() < until {
                self.metrics
                    .breaker_reject_total
                    .fetch_add(1, Ordering::Relaxed);
                return Err("redis breaker open".to_string());
            }
        }
        Ok(())
    }

    async fn record_failure(&self, fallback_counter: &AtomicU64, msg: &str) -> String {
        fallback_counter.fetch_add(1, Ordering::Relaxed);
        let mut lock = self.breaker.lock().await;
        lock.failure_count += 1;
        if lock.failure_count >= self.policy.breaker_failure_threshold {
            lock.open_until = Some(Instant::now() + self.policy.breaker_open_duration);
            self.metrics
                .breaker_open_total
                .fetch_add(1, Ordering::Relaxed);
        }
        msg.to_string()
    }

    async fn record_success(&self) {
        let mut lock = self.breaker.lock().await;
        lock.failure_count = 0;
        lock.open_until = None;
    }

    async fn with_retry<T, Fut, F>(&self, mut op: F) -> Result<T, String>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, String>>,
    {
        let attempts = self.policy.retry_attempts.max(1);
        let mut last = None;
        for i in 0..attempts {
            match timeout(self.policy.timeout, op()).await {
                Ok(Ok(v)) => return Ok(v),
                Ok(Err(e)) => last = Some(e),
                Err(_) => last = Some("redis timeout".to_string()),
            }
            if i + 1 < attempts {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        }
        Err(last.unwrap_or_else(|| "redis failure".to_string()))
    }

    pub(crate) async fn acquire_fill_lock(&self, key: &str) -> OwnedMutexGuard<()> {
        let lock = {
            let mut inflight = self.inflight.lock().await;
            Arc::clone(
                inflight
                    .entry(key.to_string())
                    .or_insert_with(|| Arc::new(Mutex::new(()))),
            )
        };
        lock.lock_owned().await
    }

    pub(crate) async fn metrics_snapshot(&self) -> RedisMetricsSnapshot {
        let tracked_keys = self.key_registry.lock().await.len() as u64;
        RedisMetricsSnapshot {
            hits: self.metrics.hits.load(Ordering::Relaxed),
            misses: self.metrics.misses.load(Ordering::Relaxed),
            read_fallbacks: self.metrics.read_fallbacks.load(Ordering::Relaxed),
            write_fallbacks: self.metrics.write_fallbacks.load(Ordering::Relaxed),
            rate_limit_fallbacks: self.metrics.rate_limit_fallbacks.load(Ordering::Relaxed),
            breaker_open_total: self.metrics.breaker_open_total.load(Ordering::Relaxed),
            breaker_reject_total: self.metrics.breaker_reject_total.load(Ordering::Relaxed),
            key_reject_total: self.metrics.key_reject_total.load(Ordering::Relaxed),
            cardinality_reject_total: self
                .metrics
                .cardinality_reject_total
                .load(Ordering::Relaxed),
            tracked_keys,
        }
    }

    pub(crate) async fn rate_limit_allow(
        &self,
        scope: &str,
        key: &str,
        cfg: &RateLimitConfig,
    ) -> Result<bool, String> {
        self.breaker_check().await?;
        let sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();
        let window_key = format!("{}:rl:{scope}:{key}:{sec}", self.prefix);
        let cap = cfg.refill_per_sec.ceil().max(1.0) as i64;
        let this = self.clone();
        let result = self
            .with_retry(move || {
                let this = this.clone();
                let window_key = window_key.clone();
                async move {
                    let mut conn = this
                        .client
                        .get_multiplexed_async_connection()
                        .await
                        .map_err(|e| e.to_string())?;
                    let count: i64 = conn
                        .incr(&window_key, 1_i64)
                        .await
                        .map_err(|e| e.to_string())?;
                    let _: bool = conn
                        .expire(&window_key, 2_i64)
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(count <= cap)
                }
            })
            .await;
        match result {
            Ok(v) => {
                self.record_success().await;
                Ok(v)
            }
            Err(e) => Err(self
                .record_failure(&self.metrics.rate_limit_fallbacks, &e)
                .await),
        }
    }

    pub(crate) async fn get_gene_cache(&self, key: &str) -> Result<Option<Vec<u8>>, String> {
        self.breaker_check().await?;
        let cache_key = format!("{}:gene:{key}", self.prefix);
        let this = self.clone();
        let result = self
            .with_retry(move || {
                let this = this.clone();
                let cache_key = cache_key.clone();
                async move {
                    let mut conn = this
                        .client
                        .get_multiplexed_async_connection()
                        .await
                        .map_err(|e| e.to_string())?;
                    conn.get(cache_key).await.map_err(|e| e.to_string())
                }
            })
            .await;
        match result {
            Ok(Some(v)) => {
                self.metrics.hits.fetch_add(1, Ordering::Relaxed);
                self.record_success().await;
                Ok(Some(v))
            }
            Ok(None) => {
                self.metrics.misses.fetch_add(1, Ordering::Relaxed);
                self.record_success().await;
                Ok(None)
            }
            Err(e) => Err(self.record_failure(&self.metrics.read_fallbacks, &e).await),
        }
    }

    pub(crate) async fn set_gene_cache(
        &self,
        key: &str,
        value: &[u8],
        ttl_secs: usize,
    ) -> Result<(), String> {
        self.breaker_check().await?;
        if key.len() > self.policy.max_key_bytes {
            self.metrics
                .key_reject_total
                .fetch_add(1, Ordering::Relaxed);
            return Err("redis key rejected by max key size policy".to_string());
        }
        let ttl = ttl_secs.clamp(1, self.policy.max_ttl_secs);
        {
            let mut keys = self.key_registry.lock().await;
            if !keys.contains(key) && keys.len() >= self.policy.max_cardinality {
                self.metrics
                    .cardinality_reject_total
                    .fetch_add(1, Ordering::Relaxed);
                return Err("redis key rejected by max cardinality policy".to_string());
            }
            keys.insert(key.to_string());
        }
        let cache_key = format!("{}:gene:{key}", self.prefix);
        let payload = value.to_vec();
        let this = self.clone();
        let result = self
            .with_retry(move || {
                let this = this.clone();
                let cache_key = cache_key.clone();
                let payload = payload.clone();
                async move {
                    let mut conn = this
                        .client
                        .get_multiplexed_async_connection()
                        .await
                        .map_err(|e| e.to_string())?;
                    let _: () = conn
                        .set_ex(cache_key, payload, ttl as u64)
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok(())
                }
            })
            .await;
        match result {
            Ok(()) => {
                self.record_success().await;
                Ok(())
            }
            Err(e) => Err(self.record_failure(&self.metrics.write_fallbacks, &e).await),
        }
    }
}
