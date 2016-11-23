use super::chrono::DateTime;
use super::chrono::UTC;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;

pub const SERVER_CONFIG: i16 = 0;
pub const CLIENT_CONFIG: i16 = 1;

fn make_path(id: i32) -> String {
    let mut path = String::from("./configs/");
    //path.push_str(&config.id.to_string());
    path + &id.to_string() + ".cfg"
}

#[derive(Serialize, Deserialize, Queryable)]
pub struct Config {
    pub id: i32,
    pub name: String,
    pub created_at: DateTime<UTC>, 
    pub config_type: i16,
}

pub fn get_all_configs(conn: &PgConnection) -> Vec<Config> {
    use super::schema::configs::dsl::*;

    configs.load::<Config>(conn).unwrap()
}

pub fn get_config(config_id: i32, conn: &PgConnection) -> Option<String> {
    use super::schema::configs::dsl::*;

    match configs.filter(id.eq(config_id)).first::<Config>(conn) {
        Ok(_) => {
            let mut file = File::open(make_path(config_id))
                .expect("Error opening config");
            let mut config = String::new();
            file.read_to_string(&mut config);
            Some(config)
        },
        _ => None
    }
}

use super::schema::configs;
#[derive(Insertable)]
#[table_name="configs"]
pub struct NewConfig<'a> {
    pub name: &'a str,
    pub created_at: DateTime<UTC>,
    pub config_type: i16,
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
