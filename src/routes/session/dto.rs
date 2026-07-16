use crate::session::{VersionMeta, VersionSnapshot};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListVersionsResponse {
    pub project_id: String,
    pub versions: Vec<VersionMeta>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVersionResponse {
    pub project_id: String,
    #[serde(flatten)]
    pub snapshot: VersionSnapshot,
}
