//! Example API with PostgreSQL users and cookie sessions.
mod api;
pub mod db;
mod error;
mod session;
pub mod state;

use axum::Router;
use state::AppState;
use tower_http::trace::TraceLayer;
use tower_sessions::{
    Expiry, MemoryStore, SessionManagerLayer,
    cookie::{SameSite, time::Duration},
};

/// Generates the spec from route registration without a database or running server.
pub fn openapi() -> utoipa::openapi::OpenApi {
    api::documented_routes().into_openapi()
}

/// Builds the example routes with a fresh session store.
pub fn router(state: AppState) -> Router {
    let sessions = SessionManagerLayer::new(MemoryStore::default())
        .with_name("api_example_session")
        .with_http_only(true)
        .with_same_site(SameSite::Strict)
        .with_always_save(true)
        .with_expiry(Expiry::OnInactivity(Duration::hours(1)));
    api::routes()
        .with_state(state)
        .layer(sessions)
        .layer(TraceLayer::new_for_http())
}
