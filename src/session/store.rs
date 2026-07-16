use crate::session::model::{VersionMeta, VersionSnapshot};

#[async_trait::async_trait]
pub trait SessionStore: Send + Sync {
    async fn append_version(&self, project_id: &str, snapshot: VersionSnapshot) -> u32;

    async fn list_versions(&self, project_id: &str) -> Vec<VersionMeta>;

    async fn get_version(&self, project_id: &str, version: u32) -> Option<VersionSnapshot>;

    async fn delete_session(&self, project_id: &str);

    async fn latest_version(&self, project_id: &str) -> u32;
}
