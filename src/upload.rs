use iron::prelude::*;
use iron::status;
use multipart::server::{Multipart, MultipartFile, MultipartData};
use regex::Regex;
use std::io::{Read, BufRead};
use std::path::PathBuf;
    
lazy_static! {
    static ref VALID_FILE: Regex = Regex::new("[0-9a-zA-Z]+").unwrap();
    static ref VALID_LINE: Regex = Regex::new(
        r#"^([a-zA-Z0-9_]+(\s+("[a-zA-Z0-9_]+"|[a-zA-Z0-9_]+))?|#[\s[:word][:punct]]*)?$"#
    ).unwrap();
}

pub fn upload_handler(req: &mut Request) -> IronResult<Response> {
    let mut multipart: Multipart<_> = match Multipart::from_request(req) {
        Ok(m) => m,
        Err(_) => return Ok(Response::with((status::BadRequest, "Request is not multipart.")))
    };
    
    let mut name: Option<String> = None;
    let mut config: Option<String> = None;

    loop {
        match multipart.read_entry() {
            Ok(Some(mut field)) => {
                if field.name == "name" && name.is_none() {
                    name = match field.data.as_text() {
                        Some(s) if VALID_FILE.is_match(s) => Some(s.into()),
                        _ => return Ok(Response::with((status::BadRequest, "Invalid file name")))
                    };
                } else if field.name == "file" {
                    let mut file = match field.data.as_file() {
                        Some(f) => f,
                        None => return Ok(Response::with((status::BadRequest, "Invalid file")))
                    };
                    for res in file.lines() {
                        match res {
                            Ok(ref line) => {
                                if line.len() > 128 || !VALID_LINE.is_match(line.as_str()) {
                                    return Ok(Response::with((status::BadRequest, "Invalid config")));
                                }
                                if let None = config {
                                    config = Some(String::new());
                                }
                                config.as_mut().unwrap().push_str(line.as_str());
                                if config.as_ref().unwrap().len() > 5000 {
                                    // Don't allow files > 10 kb for now
                                    return Ok(Response::with((status::BadRequest, "Config file too large")));
                                }
                            },
                            _ => {return Ok(Response::with((status::BadRequest, "Error while reading config")))}
                        }        
                    }
                    let mut path = String::from("./configs/");
                    path.push_str(name.as_ref().unwrap().as_str());
                    file.save_as(PathBuf::from(path.as_str()));
                }
            },
            Ok(None) => break,
            Err(_) => return Ok(Response::with((status::BadRequest, "Invalid multipart request")))
        }
    }

    if name.is_none() {
        return Ok(Response::with((status::BadRequest, "Missing config name")))
    }
    if config.is_none() {
        return Ok(Response::with((status::BadRequest, "Missing config file")))
    }

    
    
    Ok(Response::with((status::Ok, "Upload complete")))
}

// Local Variables:
// flycheck-rust-crate-type: "bin"
// End:
