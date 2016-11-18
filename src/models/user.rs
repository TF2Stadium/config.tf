use models::chrono::DateTime;
use models::chrono::UTC;
use diesel::prelude::*;

#[derive(Serialize, Deserialize, Queryable)]
pub struct User {
    pub steam_id: u64,
    pub user_name: String,
    pub created_at: DateTime<UTC>,
}
