use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use utoipa::OpenApi;

use crate::AppState;
use crate::error::{ApiErrorDetail, ApiErrorResponse};
use crate::handlers::private::AdminResponse;
use crate::types::{
  CreatePaymentData, CreatePaymentRequest, EmptyData, MobileInfo, OtpPair,
  PaymentStatusData, PaymentStatusRequest, PersonInfo, RegisterData,
  RegisterRequest, ResendRequest, VerifyRequest,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "void-srv",
        description = "void-srv sandbox environment.",
        contact(
            name = "Akilesh A S",
            email = "io@akileshas.dev",
        ),
        license(
            name = "Apache-2.0",
            url = "https://www.apache.org/licenses/LICENSE-2.0.txt",
        ),
    ),
    paths(
        crate::handlers::register::register,
        crate::handlers::verify::verify,
        crate::handlers::verify::resend,
        crate::handlers::payment::create_payment,
        crate::handlers::payment::payment_status,
    ),
    components(
        schemas(
            AdminResponse,
            ApiErrorDetail,
            ApiErrorResponse,
            CreatePaymentData,
            CreatePaymentRequest,
            EmptyData,
            MobileInfo,
            OtpPair,
            PaymentStatusData,
            PaymentStatusRequest,
            PersonInfo,
            RegisterData,
            RegisterRequest,
            ResendRequest,
            VerifyRequest,
        )
    ),
    tags(
        (name = "public", description = "endpoint for client."),
        (name = "private", description = "endpoint for admin."),
    ),
)]
pub struct ApiDoc;

pub async fn get_openapi_json() -> impl IntoResponse {
  let value = serde_json::to_value(ApiDoc::openapi()).unwrap();
  let obj = value.as_object().unwrap();

  let desired = [
    "/_/api/v2/pub/register",
    "/_/api/v2/pub/authn/verify",
    "/_/api/v2/pub/authn/resend",
    "/_/api/v2/pub/pmt/create",
    "/_/api/v2/pub/pmt/status",
  ];

  let paths_obj = obj.get("paths").unwrap().as_object().unwrap();
  let mut ordered: Vec<(String, String)> = Vec::new();
  for path in &desired {
    if let Some(v) = paths_obj.get(*path) {
      ordered.push((path.to_string(), serde_json::to_string(v).unwrap()));
    }
  }
  for (k, v) in paths_obj {
    if !desired.contains(&k.as_str()) {
      ordered.push((k.clone(), serde_json::to_string(v).unwrap()));
    }
  }

  let mut paths_json = String::from('{');
  for (i, (k, v)) in ordered.iter().enumerate() {
    if i > 0 {
      paths_json.push(',');
    }
    paths_json.push('"');
    paths_json.push_str(k);
    paths_json.push_str("\":");
    paths_json.push_str(v);
  }
  paths_json.push('}');

  let mut json = String::from('{');
  for (k, v) in obj {
    if k == "paths" {
      continue;
    }
    if json.len() > 1 {
      json.push(',');
    }
    json.push('"');
    json.push_str(k);
    json.push_str("\":");
    json.push_str(&serde_json::to_string(v).unwrap());
  }
  json.push_str(",\"paths\":");
  json.push_str(&paths_json);
  json.push('}');

  (StatusCode::OK, [("content-type", "application/json")], json)
}

pub async fn get_sandbox(State(_state): State<AppState>) -> impl IntoResponse {
  let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>void-srv-devshell</title>
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
  <style>
    html { box-sizing: border-box; overflow-y: scroll; }
    *, *:before, *:after { box-sizing: inherit; }
    body { margin: 0; background: #fafafa; }
  </style>
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
  <script>
    SwaggerUIBundle({
      url: '/sandbox/openapi.json',
      dom_id: '#swagger-ui',
      presets: [
        SwaggerUIBundle.presets.apis,
      ],
      defaultModelsExpandDepth: -1,
      docExpansion: 'list',
    });
  </script>
</body>
</html>"#.to_string();

  (
    StatusCode::OK,
    [("content-type", "text/html; charset=utf-8")],
    html,
  )
}
