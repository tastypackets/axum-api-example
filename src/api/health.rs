use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub(crate) struct Health {
    status: &'static str,
}

#[utoipa::path(get, path = "/health", tag = "health",
    description = "Checks that the server is running. Does not query the database.",
    responses((status = 200, description = "Server is running", body = Health)))]
pub(crate) async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}
