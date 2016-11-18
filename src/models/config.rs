use super::chrono::DateTime;
use super::chrono::UTC;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;

pub const SERVER_CONFIG: i16 = 0;
pub const CLIENT_CONFIG: i16 = 1;

#[derive(Serialize, Deserialize, Queryable)]
pub struct Config {
    pub id: i32,
    pub name: String,
    pub created_at: DateTime<UTC>, 
    pub config_type: i16,
    pub user_id: i64,
}

pub fn get_all_configs(conn: &PgConnection) -> Option<Vec<Config>> {
    use super::schema::configs::dsl::*;

    configs.load::<Config>(conn).ok()
}

use super::schema::configs;
#[derive(Insertable)]
#[table_name="configs"]
pub struct NewConfig<'a> {
    pub name: &'a str,
    pub created_at: DateTime<UTC>,
    pub config_type: i16,
    pub user_id: i64
}

impl<'a> NewConfig<'a> {
    pub fn save(self, config_str: &str, conn: &PgConnection) -> Config {
        let config: Config = diesel::insert(&self)
            .into(configs::table)
            .get_result(conn)
            .expect("Error saving new config record");

        let mut path = String::from("./configs/");
        path.push_str(&config.id.to_string());
        path += ".cfg";

        let mut file: File = OpenOptions::new().write(true)
            .create_new(true)
            .open(path)
            .expect("opening file");
        file.write_all(config_str.as_bytes()).expect("writing config to file");
        config
    }
}
