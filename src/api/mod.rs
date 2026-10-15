mod auth;
mod health;
mod openapi;
mod users;

use crate::state::AppState;
use axum::{Json, Router, routing::get};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

pub(crate) fn routes() -> Router<AppState> {
    let api = OpenApiRouter::new()
        .nest("/auth", auth::routes())
        .nest("/users", users::routes());

    let (router, spec) = OpenApiRouter::with_openapi(openapi::ApiDoc::openapi())
        .routes(routes!(health::health))
        .nest("/api", api)
        .split_for_parts();

    router.route("/api/openapi.json", get(move || async move { Json(spec) }))
}
