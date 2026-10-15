mod login;
mod logout;

use crate::state::AppState;
use utoipa_axum::{router::OpenApiRouter, routes};

pub(super) fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(login::login))
        .routes(routes!(logout::logout))
}
