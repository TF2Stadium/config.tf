#[macro_use(itry)]
extern crate iron;
extern crate hyper;
extern crate multipart;
extern crate router;
extern crate dotenv;
extern crate tempdir;
extern crate regex;
extern crate sha1;
extern crate rusqlite;
#[macro_use]
extern crate lazy_static;

use std::io::prelude::*;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::fs;
use std::path::{Path};
use std::io::{BufReader};
use std::env;
use std::io;
use tempdir::TempDir;
use hyper::header::{ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use multipart::server::{Multipart, SavedFile};
use iron::prelude::*;
use iron::{status, BeforeMiddleware, typemap, TypeMap};
use router::Router;
use dotenv::dotenv;
use regex::Regex;
use sha1::Sha1;
use rusqlite::Connection;

fn sha1(s: &[u8]) -> String {
    let mut digest = Sha1::new();
    digest.update(s);
    return digest.digest().to_string();
}

fn ensure_ends_with(suffix: &str, s: String) -> String {
    if !s.ends_with(suffix) {
        s + suffix
    } else {
        s
    }
}

fn config_name_to_file_name(s: String) -> String { format!("./configs/{}", sha1(ensure_ends_with(".cfg", s).as_bytes())) }

lazy_static! {
    static ref VALID_LINE: Regex = Regex::new(
        r#"^([a-zA-Z0-9_]+(\s+("[a-zA-Z\.0-9_]+"|[a-zA-Z\.0-9_]+))?|#[\s[:word][:punct]]*)?$"#
    ).unwrap();

    static ref VALID_NAME: Regex = Regex::new(
        r#"^[a-zA-Z0-9-_]+(\.cfg)?$"#
    ).unwrap();
}

static INDEX: &'static [u8] = include_bytes!("index.html");
fn index_handler(_: &mut Request) -> IronResult<Response> {
    let mut response = Response::with((status::Ok, INDEX));
    response.headers.set(
        ContentType(Mime(TopLevel::Text, SubLevel::Html,
                         vec![(Attr::Charset, Value::Utf8)]))
    );
    Ok(response)
}

enum ValidateConfigError {
    Io(io::Error),
    Validation(String),
}
fn validate_config_at(p: &Path) -> Result<(), ValidateConfigError> {
    match File::open(p) {
        Ok(f) => {
            for l in BufReader::new(&f).lines() {
                let line = l.unwrap();
                if line.len() > 128 {
                    return Err(ValidateConfigError::Validation(format!("line too long: \"{}\"", line)));
                }
                if !VALID_LINE.is_match(line.as_str()) {
                    return Err(ValidateConfigError::Validation(format!("invalid line: \"{}\"", line)));
                }
            }
            Ok(())
        },
        Err(e) => Err(ValidateConfigError::Io(e))
    }
}

fn upload_handler(req: &mut iron::Request) -> IronResult<Response> {
    let upload_dir = TempDir::new("configtf-upload").unwrap();

    // pull the extensions out of the request object before handing it
    // over to multipart's ownership (multipart only exposes the
    // request's _body_ back to us...)
    let extensions = std::mem::replace(&mut req.extensions, TypeMap::new());

    let mut multipart = match Multipart::from_request(req) {
        Ok(multipart) => multipart,
        Err(_) => return Ok(Response::with((status::BadRequest, "The request is not multipart"))),
    };

    let mut name: Option<String> = None;
    let mut file: Option<io::Result<SavedFile>> = None;
    match multipart.foreach_entry(|mut field| {
        if field.name == "name" && name.is_none() {
            name = Some(field.data.as_text().unwrap_or("").to_string());
        } else if field.name == "file" && file.is_none() {
            file = Some(
                field.data.as_file().unwrap().save_in_limited(upload_dir.path(), 30000)
            );
        }
    }) {
        Ok(_) => (),
        Err(_) => return Ok(Response::with((status::BadRequest, "Error uploading"))),
    }

    if !name.is_some() || !file.is_some() {
        return Ok(Response::with((status::BadRequest, "Name and file required")));
    }

    let given_name = name.unwrap();
    let save_status = file.unwrap();
    if save_status.is_err() {
        println!("error saving {}", save_status.unwrap_err());
        return Ok(Response::with((status::InternalServerError, "Error uploading")));
    }
    let saved = save_status.unwrap();

    if !VALID_NAME.is_match(given_name.as_str()) {
        return Ok(Response::with((status::BadRequest, "Invalid config name, can only include: a-z, A-Z, 0-9, -, _ (and may end with .cfg)")));
    }

    let dest_path = config_name_to_file_name(given_name.clone());
    if Path::new(&dest_path).exists() {
        return Ok(Response::with((status::BadRequest, "Config with that name already exists")));
    }

    // TODO: Could do this validation with the temp file shenanigans
    match validate_config_at(&saved.path) {
        Ok(()) => {
            // Instead of a rename, copy then delete the old one (because the uploaded file
            // is saved to a tmp dir, which is often on a different filesystem, so rename
            // doesn't work)
            let db_mutex = extensions.get::<DbConn>().unwrap();
            let conn = db_mutex.lock().unwrap();
            conn.execute("INSERT INTO config VALUES (?)", &[&given_name]).unwrap();
            match fs::copy(&saved.path, dest_path) {
                Ok(_) => {
                    let _ = fs::remove_file(&saved.path);
                    Ok(Response::with((status::Ok, "Upload complete")))
                },
                Err(_) => {
                    Ok(Response::with((status::InternalServerError, "Error publishing")))
                }
            }

        },
        Err(ValidateConfigError::Validation(reason)) =>
            Ok(Response::with((status::BadRequest,
                               format!("Invalid config file\n{}", reason)))),
        Err(ValidateConfigError::Io(_)) => {
            Ok(Response::with((status::InternalServerError, "Error uploading")))
        }
    }
}

fn config_handler(req: &mut Request) -> IronResult<Response> {
    let config = req.extensions.get::<Router>().unwrap().find("config").unwrap();

    if !VALID_NAME.is_match(config) {
        return Ok(Response::with((status::BadRequest, "Invalid config name")));
    }

    if let Ok(ref mut f) = File::open(config_name_to_file_name(config.to_string())) {
        let mut contents = String::new();
        match f.read_to_string(&mut contents) {
            Ok(_) => Ok(Response::with((status::Ok, contents))),
            Err(_) => Ok(Response::with((status::Ok, contents))),
        }
    } else {
        Ok(Response::with((status::NotFound, "Not Found")))
    }
}

static MIGRATIONS: [&'static str; 1] = [
    include_str!("./migrations/0.sql")
];

fn db_version(conn: &Connection) -> rusqlite::Result<i64> {
    let mut cnt_tables_query = try!(conn.prepare("SELECT count(*) FROM sqlite_master"));
    let mut cnt_tables = try!(cnt_tables_query.query_map(&[], |row| row.get(0)));
    let cnt_table = cnt_tables.nth(0).unwrap_or(Ok(0)).unwrap();

    if cnt_table > 0 {
        conn.query_row("SELECT db_version FROM db_version", &[], |row| row.get::<i32, i64>(0))
    } else {
        Ok(-1)
    }
}

fn run_migration(conn: &Connection, i: usize) -> rusqlite::Result<()> {
    try!(conn.execute_batch(MIGRATIONS[i]));
    try!(conn.execute("UPDATE db_version SET db_version=?", &[&(i as i32)]));
    Ok(())
}

fn setup_db(conn: &Connection) -> rusqlite::Result<()> {
    let version = try!(db_version(conn));

    for i in (version + 1)..(MIGRATIONS.len() as i64) {
        println!("Running DB migration: {}", i);
        try!(run_migration(conn, i as usize));
    }

    Ok(())
}

struct DbConn { conn: Arc<Mutex<Connection>>, }

impl typemap::Key for DbConn { type Value = Arc<Mutex<Connection>>; }

impl BeforeMiddleware for DbConn {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<DbConn>(self.conn.clone());
        Ok(())
    }
}

fn main() {
    dotenv().ok();
    let port: u16 = env::var("PORT").unwrap_or("".to_string()).parse().unwrap_or(3000);
    let db_path: String = env::var("SQLITE_DB").unwrap_or("./db.sqlite".to_string());

    let conn = Connection::open(db_path).unwrap();
    setup_db(&conn).unwrap();

    let mut router = Router::new();
    router.get("/", index_handler, "index");
    router.post("/cfg", upload_handler, "upload");
    router.get("/cfg/*config", config_handler, "config");

    let mut chain = Chain::new(router);
    chain.link_before(DbConn {conn: Arc::new(Mutex::new(conn))});

    let _server = Iron::new(chain).http(("0.0.0.0", port)).unwrap();
    println!("Listening on port {}", port);
}

// Local Variables:
// flycheck-rust-crate-type: "bin"
// End:
