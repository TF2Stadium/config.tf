use iron::prelude::*;
use iron::Headers;
use iron::headers::{ContentDisposition, DispositionType, Charset, DispositionParam, ContentType};
use iron::status;
use serde_json;
use models::config;
use conn::*;

pub fn get_all_configs(_: &mut Request) -> IronResult<Response> {
    let configs = config::get_all_configs(&establish_connection());
    let mut headers = Headers::new();
    let mut res = Response::with((status::Ok,
                              serde_json::to_string(&configs).unwrap()));

    headers.set(ContentType::json());
    res.headers = headers;
    Ok(res)
}

pub fn get_config(req: &mut Request) -> IronResult<Response> {
    let url = req.url.clone().into_generic_url();
    let pairs = url.query_pairs();
    let mut id: i32 = 0;

    for (key, value) in pairs {
        if key == "id" {
            id = match i32::from_str_radix(&value, 10) {
                Ok(n) => n,
                Err(_) => return Ok(Response::with((status::BadRequest, "Invalid ID")))
            };
        }
    };

    let config_str: String = match config::get_config(id, &establish_connection()) {
        Some(config_str) => {
            config_str
        }
        None => return Ok(Response::with((status::NotFound, "Couldn't find config"))),
    };

    let mut headers = Headers::new();
    headers.set(ContentDisposition{
        disposition: DispositionType::Attachment,
        parameters: vec![DispositionParam::Filename(
            Charset::Iso_8859_1, // The character set for the bytes of the filename
            None, // The optional language tag (see `language-tag` crate)
            config_str.into_bytes())
        ]});

    let mut res = Response::new();
    res.status = Some(status::Ok);
    res.headers = headers;

    Ok(res)
}
