use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Sliding-window rate limiter keyed by IP address.
///
/// Each entry stores the timestamps of recent attempts. When the count
/// within the window reaches `max_attempts`, further attempts are rejected
/// until enough of the oldest timestamps fall out of the window.
///
/// Cloning is cheap — the internal map is behind an `Arc`. All clones
/// share the same state.
#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<RwLock<HashMap<IpAddr, Vec<Instant>>>>,
    max_attempts: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_attempts: usize, window: Duration) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            max_attempts,
            window,
        }
    }

    /// Record an attempt from `ip`. Returns `Ok(())` if allowed, or
    /// `Err(retry_after)` if the limit has been reached.
    pub async fn check_and_record(&self, ip: IpAddr) -> Result<(), Duration> {
        let now = Instant::now();
        let cutoff = now.checked_sub(self.window).unwrap_or(now);

        let mut map = self.inner.write().await;
        let entry = map.entry(ip).or_default();

        // Drop attempts that fell outside the window.
        entry.retain(|t| *t > cutoff);

        if entry.len() >= self.max_attempts {
            // The oldest attempt determines when the window frees up.
            let oldest = entry[0];
            let retry_after = self
                .window
                .checked_sub(now - oldest)
                .unwrap_or(Duration::from_secs(1));
            return Err(retry_after);
        }

        entry.push(now);
        Ok(())
    }

    /// Clear attempts for `ip`. Called after a successful login so that
    /// legitimate users aren't penalized for past typos.
    pub async fn clear(&self, ip: IpAddr) {
        let mut map = self.inner.write().await;
        map.remove(&ip);
    }

    /// Remove entries that are entirely outside the window. Called by a
    /// background task so the map doesn't grow unbounded.
    pub async fn cleanup(&self) {
        let cutoff = Instant::now().checked_sub(self.window).unwrap_or_else(Instant::now);
        let mut map = self.inner.write().await;
        map.retain(|_, v| {
            v.retain(|t| *t > cutoff);
            !v.is_empty()
        });
    }

    /// Current number of tracked IPs. Used by the cleanup task for logging.
    pub async fn size(&self) -> usize {
        self.inner.read().await.len()
    }
}
