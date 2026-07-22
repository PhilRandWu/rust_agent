use super::dto::HealthResponse;
use axum::Json;
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}
