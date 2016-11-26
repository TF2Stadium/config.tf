#[macro_use(itry)]
extern crate iron;
extern crate hyper;
extern crate multipart;
extern crate router;
extern crate dotenv;
extern crate tempdir;
extern crate regex;
extern crate sha1;
#[macro_use]
extern crate lazy_static;

use std::io::prelude::*;
use std::fs::File;
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
use iron::status;
use router::Router;
use dotenv::dotenv;
use regex::Regex;
use sha1::Sha1;

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
    match Multipart::from_request(req) {
        Ok(mut multipart) => {
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
                Ok(_) => {
                    if name.is_some() && file.is_some() {
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

                        let dest_path = config_name_to_file_name(given_name);
                        if Path::new(&dest_path).exists() {
                            return Ok(Response::with((status::BadRequest, "Config with that name already exists")));
                        }

                        // TODO: Could do all this reading without the temp file
                        match validate_config_at(&saved.path) {
                            Ok(()) => {
                                // Instead of a rename, copy then delete the old one (because the uploaded file
                                // is saved to a tmp dir, which is often on a different filesystem, so rename
                                // doesn't work)
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
                    } else {
                        Ok(Response::with((status::BadRequest, "Name and file required")))
                    }
                },
                Err(_) => Ok(Response::with((status::BadRequest, "Error uploading")))
            }
        }
        Err(_) => {
            Ok(Response::with((status::BadRequest, "The request is not multipart")))
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

fn main() {
    dotenv().ok();

    let mut router = Router::new();
    router.get("/", index_handler, "index");
    router.post("/cfg", upload_handler, "upload");
    router.get("/cfg/*config", config_handler, "config");

    let port: u16 = env::var("PORT").unwrap_or("".to_string()).parse().unwrap_or(3000);
    let _server = Iron::new(router).http(("0.0.0.0", port)).unwrap();
    println!("Listening on port {}", port);
}

// Local Variables:
// flycheck-rust-crate-type: "bin"
// End:
