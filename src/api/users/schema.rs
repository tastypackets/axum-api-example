use serde::Serialize;
use utoipa::ToSchema;

use crate::db;

/// User object, this rustdoc comment will appear in openapi spec
#[derive(Serialize, ToSchema)]
pub(crate) struct UserResponse {
    /// User id, this rustdoc comment will appear in openapi spec
    pub id: i64,
    pub username: String,
    pub display_name: String,
}

// Notice private_notes is not in UserResponse struct and not used in this conversion
// so the field is always dropped from API responses and not exposed to the public
impl From<db::UserRecord> for UserResponse {
    fn from(user: db::UserRecord) -> Self {
        Self {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
        }
    }
}
