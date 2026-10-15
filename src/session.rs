use crate::{db, error::ApiError};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use sqlx::PgPool;
use tower_sessions::Session;

pub(crate) const USER_ID: &str = "user_id";
pub(crate) struct SignedIn(pub db::UserRecord);

impl<S> FromRequestParts<S> for SignedIn
where
    S: Send + Sync,
    PgPool: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|error| {
                tracing::error!(?error, "session middleware missing");
                ApiError::Internal
            })?;
        let id = session
            .get::<i64>(USER_ID)
            .await?
            .ok_or(ApiError::Unauthorized)?;
        // This example loads a current user for each protected request. Other
        // designs may avoid this read, such as a verified JWT carrying user data.
        let pool = PgPool::from_ref(state);
        let user = db::user_by_id(&pool, id)
            .await?
            .ok_or(ApiError::Unauthorized)?;
        Ok(Self(user))
    }
}
