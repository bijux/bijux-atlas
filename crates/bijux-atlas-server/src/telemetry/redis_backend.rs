use crate::RateLimitConfig;
use redis::AsyncCommands;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub(crate) struct RedisBackend {
    client: redis::Client,
    prefix: String,
}

impl RedisBackend {
    pub(crate) fn new(url: &str, prefix: &str) -> Result<Self, String> {
        let client = redis::Client::open(url).map_err(|e| e.to_string())?;
        Ok(Self {
            client,
            prefix: prefix.to_string(),
        })
    }

    pub(crate) async fn rate_limit_allow(
        &self,
        scope: &str,
        key: &str,
        cfg: &RateLimitConfig,
    ) -> Result<bool, String> {
        // Shared fixed-window per-second limiter for cross-pod consistency.
        let sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();
        let window_key = format!("{}:rl:{scope}:{key}:{sec}", self.prefix);
        let cap = cfg.refill_per_sec.ceil().max(1.0) as i64;
        let mut conn = self
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

    pub(crate) async fn get_gene_cache(&self, key: &str) -> Result<Option<Vec<u8>>, String> {
        let cache_key = format!("{}:gene:{key}", self.prefix);
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| e.to_string())?;
        conn.get(cache_key).await.map_err(|e| e.to_string())
    }

    pub(crate) async fn set_gene_cache(
        &self,
        key: &str,
        value: &[u8],
        ttl_secs: usize,
    ) -> Result<(), String> {
        let cache_key = format!("{}:gene:{key}", self.prefix);
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| e.to_string())?;
        let _: () = conn
            .set_ex(cache_key, value, ttl_secs as u64)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
