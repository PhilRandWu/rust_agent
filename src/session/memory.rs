use crate::session::config::SessionConfig;
use crate::session::model::{VersionMeta, VersionSnapshot};
use crate::session::store::SessionStore;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct SessionState {
    latest_version: u32,
    versions: VecDeque<VersionSnapshot>,
}

impl SessionState {
    fn new() -> Self {
        Self {
            latest_version: 0,
            versions: VecDeque::new(),
        }
    }
}

pub struct MemorySessionStore {
    inner: Arc<RwLock<Inner>>,
    config: SessionConfig,
}

struct Inner {
    projects: HashMap<String, SessionState>,
    lru: VecDeque<String>,
}

impl MemorySessionStore {
    pub fn new(config: SessionConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner {
                projects: HashMap::new(),
                lru: VecDeque::new(),
            })),
            config,
        }
    }

    fn touch(lru: &mut VecDeque<String>, project_id: &str) {
        if let Some(pos) = lru.iter().position(|p| p == project_id) {
            lru.remove(pos);
        }
        lru.push_back(project_id.to_string());
    }

    fn touch_new(inner: &mut Inner, project_id: &str, cap: usize) {
        while inner.lru.len() >= cap {
            if let Some(evict) = inner.lru.pop_front() {
                inner.projects.remove(&evict);
                tracing::debug!(target: "session.lru", evicted = %evict, "evicted project (global cap)");
            } else {
                break;
            }
        }
        inner.lru.push_back(project_id.to_string());
    }
}

#[async_trait::async_trait]
impl SessionStore for MemorySessionStore {
    async fn append_version(&self, project_id: &str, mut snapshot: VersionSnapshot) -> u32 {
        let mut guard = self.inner.write().await;
        let existed = guard.projects.contains_key(project_id);
        if !existed {
            Self::touch_new(&mut guard, project_id, self.config.global_cap);
            guard
                .projects
                .insert(project_id.to_string(), SessionState::new());
        } else {
            Self::touch(&mut guard.lru, project_id);
        }

        let state = guard
            .projects
            .get_mut(project_id)
            .expect("just inserted / verified above");
        state.latest_version += 1;
        snapshot.version = state.latest_version;

        state.versions.push_back(snapshot);
        while state.versions.len() > self.config.per_project_cap {
            let dropped = state.versions.pop_front();
            if let Some(d) = dropped {
                tracing::debug!(
                    target: "session.lru",
                    project_id = %project_id,
                    dropped_version = d.version,
                    "dropped old version (per-project cap)"
                );
            }
        }
        state.latest_version
    }

    async fn list_versions(&self, project_id: &str) -> Vec<VersionMeta> {
        let mut guard = self.inner.write().await;
        if !guard.projects.contains_key(project_id) {
            return Vec::new();
        }
        Self::touch(&mut guard.lru, project_id);
        let state = guard.projects.get(project_id).unwrap();
        state.versions.iter().map(VersionMeta::from).collect()
    }

    async fn get_version(&self, project_id: &str, version: u32) -> Option<VersionSnapshot> {
        let mut guard = self.inner.write().await;
        if !guard.projects.contains_key(project_id) {
            return None;
        }
        Self::touch(&mut guard.lru, project_id);
        let state = guard.projects.get(project_id).unwrap();
        state
            .versions
            .iter()
            .find(|s| s.version == version)
            .cloned()
    }

    async fn delete_session(&self, project_id: &str) {
        let mut guard = self.inner.write().await;
        guard.projects.remove(project_id);
        if let Some(pos) = guard.lru.iter().position(|p| p == project_id) {
            guard.lru.remove(pos);
        }
    }

    async fn latest_version(&self, project_id: &str) -> u32 {
        let guard = self.inner.read().await;
        guard
            .projects
            .get(project_id)
            .map(|s| s.latest_version)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::model::Operation;
    use std::collections::BTreeMap;

    fn mk_snapshot(prompt: &str) -> VersionSnapshot {
        VersionSnapshot {
            version: 0,
            timestamp_ms: 1,
            operation: Operation::Create,
            prompt: prompt.into(),
            file_count: 1,
            files: BTreeMap::from([("/a.tsx".into(), "x".into())]),
        }
    }

    #[tokio::test]
    async fn append_and_list_returns_versions_in_order() {
        let store = MemorySessionStore::new(SessionConfig::default());
        let v1 = store.append_version("p1", mk_snapshot("first")).await;
        let v2 = store.append_version("p1", mk_snapshot("second")).await;
        assert_eq!((v1, v2), (1, 2));

        let list = store.list_versions("p1").await;
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].version, 1);
        assert_eq!(list[1].version, 2);
        assert_eq!(list[0].prompt, "first");
    }

    #[tokio::test]
    async fn get_version_returns_full_snapshot_and_missing_returns_none() {
        let store = MemorySessionStore::new(SessionConfig::default());
        store.append_version("p1", mk_snapshot("hi")).await;
        let got = store.get_version("p1", 1).await.expect("should exist");
        assert_eq!(got.files.len(), 1);
        assert!(store.get_version("p1", 99).await.is_none());
        assert!(store.get_version("nope", 1).await.is_none());
    }

    #[tokio::test]
    async fn per_project_cap_drops_oldest_but_latest_version_keeps_growing() {
        let cfg = SessionConfig {
            per_project_cap: 2,
            global_cap: 10,
        };
        let store = MemorySessionStore::new(cfg);
        store.append_version("p", mk_snapshot("a")).await;
        store.append_version("p", mk_snapshot("b")).await;
        store.append_version("p", mk_snapshot("c")).await;

        let list = store.list_versions("p").await;
        assert_eq!(list.len(), 2, "cap=2 → only 2 kept");
        assert_eq!(list[0].version, 2, "v1 dropped, v2 is now oldest");
        assert_eq!(list[1].version, 3);
        assert_eq!(store.latest_version("p").await, 3);
    }

    #[tokio::test]
    async fn global_cap_evicts_least_recently_used_project() {
        let cfg = SessionConfig {
            per_project_cap: 5,
            global_cap: 2,
        };
        let store = MemorySessionStore::new(cfg);

        store.append_version("p1", mk_snapshot("a")).await;
        store.append_version("p2", mk_snapshot("b")).await;
        // Touch p1 -> p2 becomes LRU
        let _ = store.list_versions("p1").await;
        // Insert p3 -> should evict p2
        store.append_version("p3", mk_snapshot("c")).await;

        assert_eq!(store.latest_version("p1").await, 1);
        assert_eq!(store.latest_version("p2").await, 0, "p2 evicted");
        assert_eq!(store.latest_version("p3").await, 1);
    }

    #[tokio::test]
    async fn delete_session_is_idempotent() {
        let store = MemorySessionStore::new(SessionConfig::default());
        store.append_version("p", mk_snapshot("x")).await;
        store.delete_session("p").await;
        store.delete_session("p").await; // no panic
        assert_eq!(store.latest_version("p").await, 0);
        assert!(store.list_versions("p").await.is_empty());
    }
}
