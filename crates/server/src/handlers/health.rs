use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

use crate::AppState;

pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
  match sqlx::query("SELECT 1").fetch_one(&state.pool).await {
    Ok(_) => (StatusCode::OK, Json(json!({"status": "ok"}))).into_response(),
    Err(e) => {
      tracing::error!(error = %e, "health check: database unreachable");
      (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({"status": "degraded", "database": "unreachable"})),
      )
        .into_response()
    }
  }
}
