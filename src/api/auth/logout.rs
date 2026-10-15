use crate::{
    error::{ApiError, ErrorBody},
    session::SignedIn,
};
use axum::http::StatusCode;
use tower_sessions::Session;

#[utoipa::path(post, path = "/logout", tag = "auth",
    description = "Deletes the current session.",
    security(("session" = [])),
    responses(
        (status = 204, description = "Signed out"),
        (status = 401, description = "Not signed in or user deleted", body = ErrorBody),
        (status = 500, description = "Database or session failure", body = ErrorBody)
    ))]
pub(crate) async fn logout(_user: SignedIn, session: Session) -> Result<StatusCode, ApiError> {
    session.flush().await?;
    Ok(StatusCode::NO_CONTENT)
}
