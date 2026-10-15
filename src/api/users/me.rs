use super::schema::UserResponse;
use crate::{error::ErrorBody, session::SignedIn};
use axum::{Json, http::header::CACHE_CONTROL};

#[utoipa::path(get, path = "/me", tag = "users",
    description = "Returns the signed-in user from PostgreSQL.",
    security(("session" = [])),
    responses(
        (status = 200, description = "Current user", body = UserResponse),
        (status = 401, description = "Not signed in or user deleted", body = ErrorBody),
        (status = 500, description = "Database or session failure", body = ErrorBody)
    ))]
pub(crate) async fn me(SignedIn(user): SignedIn) -> impl axum::response::IntoResponse {
    (
        [(CACHE_CONTROL, "no-store")],
        Json(UserResponse::from(user)),
    )
}
