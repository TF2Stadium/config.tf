use chrono::NaiveTime;
use serde_json;
use diesel::prelude::*;
use diesel::types::Time;

#[derive(ToSql, FromSql)]
pub enum ConfigType {
    Server = 0,
    Client = 1
}

#[derive(Serialize, Deserialize, Queryable)]
pub struct Config {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveTime, 
    pub config_type: ConfigType,
    pub config_path: String,
    pub user_id: u64,
}

#[derive(Serialize, Deserialize, Queryable)]
pub struct User {
    pub steam_id: u64,
    pub user_name: String,
    pub created_at: NaiveTime,
}

// Local Variables:
// flycheck-rust-crate-type: "bin"
// End:
