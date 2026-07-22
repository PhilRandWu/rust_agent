use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

#[derive(Debug, Clone)]
pub struct LlmGate {
    inner: Arc<Semaphore>,
    capacity: usize,
}

impl LlmGate {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity >= 1, "LlmGate capacity must be >= 1");
        Self {
            inner: Arc::new(Semaphore::new(capacity)),
            capacity,
        }
    }

    pub fn default_capacity() -> usize {
        std::cmp::min(4, std::cmp::max(1, num_cpus::get()))
    }

    pub async fn acquire(&self) -> anyhow::Result<OwnedSemaphorePermit> {
        let start = std::time::Instant::now();
        let available_before = self.inner.available_permits();
        let permit = self.inner.clone().acquire_owned().await?;
        let wait_ms = start.elapsed().as_millis();
        if wait_ms > 50 {
            tracing::debug!(
                target: "llm.gate",
                wait_ms,
                available_before,
                capacity = self.capacity,
                "gate permit acquired (waited)"
            );
        }
        Ok(permit)
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn available(&self) -> usize {
        self.inner.available_permits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;
    use tokio::task::JoinSet;

    #[tokio::test]
    async fn gate_enforces_capacity() {
        let gate = LlmGate::new(2);
        let concurrent = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));

        let mut tasks = JoinSet::new();
        for _ in 0..10 {
            let gate = gate.clone();
            let c = Arc::clone(&concurrent);
            let p = Arc::clone(&peak);
            tasks.spawn(async move {
                let _permit = gate.acquire().await.unwrap();
                let now = c.fetch_add(1, Ordering::SeqCst) + 1;
                p.fetch_max(now, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(20)).await;
                c.fetch_sub(1, Ordering::SeqCst);
            });
        }
        while tasks.join_next().await.is_some() {}

        assert!(
            peak.load(Ordering::SeqCst) <= 2,
            "peak should not exceed capacity"
        );
    }

    #[test]
    fn default_capacity_is_bounded() {
        let cap = LlmGate::default_capacity();
        assert!(cap >= 1 && cap <= 4);
    }
}
