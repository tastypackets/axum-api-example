mod auth;
mod health;
mod openapi;
mod users;

use crate::state::AppState;
use axum::{Json, Router, routing::get};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

pub(crate) fn documented_routes() -> OpenApiRouter<AppState> {
    let api = OpenApiRouter::new()
        .nest("/auth", auth::routes())
        .nest("/users", users::routes());

    OpenApiRouter::with_openapi(openapi::ApiDoc::openapi())
        .routes(routes!(health::health))
        .nest("/api", api)
}

pub(crate) fn routes() -> Router<AppState> {
    let (router, spec) = documented_routes().split_for_parts();

    // Release builds omit this endpoint to avoid publishing the API spec by default.
    // Remove this condition to serve the spec in production too.
    if cfg!(debug_assertions) {
        router.route("/api/openapi.json", get(move || async move { Json(spec) }))
    } else {
        router
    }
}
