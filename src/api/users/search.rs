use super::schema::UserResponse;
use crate::{
    db,
    error::{ApiError, ErrorBody},
    session::SignedIn,
};
use axum::{
    Json,
    extract::{Query, State, rejection::QueryRejection},
};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub(crate) struct SearchUsers {
    /// Username or display name substring, ASCII case-insensitive, at most 100 bytes after trimming.
    q: String,
}

#[utoipa::path(get, path = "/search", tag = "users",
    description = "Searches usernames and display names, sorted by username, up to 50 results.",
    params(SearchUsers),
    security(("session" = [])),
    responses(
        (status = 200, description = "Matching users", body = [UserResponse]),
        (status = 400, description = "Missing or invalid query", body = ErrorBody),
        (status = 401, description = "Not signed in or user deleted", body = ErrorBody),
        (status = 500, description = "Database or session failure", body = ErrorBody)
    ))]
pub(crate) async fn search_users(
    _user: SignedIn,
    State(pool): State<PgPool>,
    query: Result<Query<SearchUsers>, QueryRejection>,
) -> Result<Json<Vec<UserResponse>>, ApiError> {
    let Query(search) = query.map_err(|error| ApiError::BadRequest(error.body_text()))?;
    let query = search.q.trim();
    if query.is_empty() || query.len() > 100 {
        return Err(ApiError::BadRequest(
            "q must contain 1 to 100 bytes after trimming".into(),
        ));
    }
    Ok(Json(
        db::search(&pool, query)
            .await?
            .into_iter()
            .map(UserResponse::from)
            .collect(),
    ))
}
