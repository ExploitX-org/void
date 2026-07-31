use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminResponse {
  pub status: String,
  pub message: String,
}
