//! PostgreSQL setup and user queries.
use sqlx::{FromRow, PgPool, postgres::PgPoolOptions};

/// A user stored in PostgreSQL.
// Serialize and ToSchema belong on the API type, which excludes private fields.
#[derive(FromRow)]
pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub private_notes: String,
}

/// Opens PostgreSQL and applies the bundled schema and example data.
pub async fn connect(url: &str) -> Result<PgPool, Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;
    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
}

pub(crate) async fn user_by_id(pool: &PgPool, id: i64) -> Result<Option<UserRecord>, sqlx::Error> {
    sqlx::query_as("SELECT id, username, display_name, private_notes FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub(crate) async fn user_by_username(
    pool: &PgPool,
    name: &str,
) -> Result<Option<UserRecord>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, username, display_name, private_notes FROM users WHERE username = $1",
    )
    .bind(name)
    .fetch_optional(pool)
    .await
}

pub(crate) async fn search(pool: &PgPool, query: &str) -> Result<Vec<UserRecord>, sqlx::Error> {
    // strpos treats % and _ as literal characters, unlike LIKE.
    sqlx::query_as(
        r#"SELECT id, username, display_name, private_notes FROM users
        WHERE strpos(lower(username COLLATE "C"), lower($1 COLLATE "C")) > 0
           OR strpos(lower(display_name COLLATE "C"), lower($1 COLLATE "C")) > 0
        ORDER BY username LIMIT 50"#,
    )
    .bind(query)
    .fetch_all(pool)
    .await
}
