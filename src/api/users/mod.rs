mod me;
pub(super) mod schema;
mod search;

use crate::state::AppState;
use utoipa_axum::{router::OpenApiRouter, routes};

pub(super) fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(me::me))
        .routes(routes!(search::search_users))
}
