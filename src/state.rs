//! Shared application dependencies.

use axum::extract::FromRef;
use sqlx::PgPool;

/// Dependencies available to handlers through `State`.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}
