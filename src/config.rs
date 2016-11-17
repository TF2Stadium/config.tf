extern crate chrono;

use self::chrono::DateTime;
use self::chrono::UTC;
use serde_json;

pub enum ConfigType {
    Server,
    Client
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub uploaded_on: DateTime<UTC>, 
    pub config_type: ConfigType,
    pub path: String
}
