use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    http::header::CACHE_CONTROL,
};
use serde::Deserialize;
use sqlx::PgPool;
use tower_sessions::Session;
use utoipa::ToSchema;

use crate::{
    api::users::schema::UserResponse,
    db,
    error::{ApiError, ErrorBody},
    session::USER_ID,
};

#[derive(Deserialize, ToSchema)]
pub(crate) struct Login {
    /// Demo username, always `admin`.
    username: String,
    /// Demo password, always `admin`.
    password: String,
}

#[utoipa::path(post, path = "/login", tag = "auth",
    description = "Starts a session with the demo credentials admin/admin.",
    request_body = Login,
    responses(
        (status = 200, description = "Signed in, session cookie set", body = UserResponse),
        (status = 400, description = "Invalid JSON body", body = ErrorBody),
        (status = 401, description = "Invalid credentials", body = ErrorBody),
        (status = 500, description = "Database or session failure", body = ErrorBody)
    ))]
pub(crate) async fn login(
    State(pool): State<PgPool>,
    session: Session,
    body: Result<Json<Login>, JsonRejection>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
    let Json(credentials) = body.map_err(|error| ApiError::BadRequest(error.body_text()))?;
    if credentials.username != "admin" || credentials.password != "admin" {
        return Err(ApiError::Unauthorized);
    }
    let user = db::user_by_username(&pool, &credentials.username)
        .await?
        .ok_or(ApiError::Unauthorized)?;
    // Rotate before granting access so an earlier session ID cannot keep it.
    session.cycle_id().await?;
    session.insert(USER_ID, user.id).await?;
    Ok((
        [(CACHE_CONTROL, "no-store")],
        Json(UserResponse::from(user)),
    ))
}
