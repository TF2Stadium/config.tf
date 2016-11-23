#![feature(custom_derive, proc_macro)]
#[macro_use(itry)]
#[macro_use(iexpect)]
extern crate iron;
extern crate hyper;
extern crate multipart;
extern crate router;
extern crate dotenv;
extern crate tempdir;
extern crate regex;
extern crate sha1;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate diesel_codegen;
#[macro_use] extern crate log;
extern crate env_logger;
#[macro_use]
extern crate serde_derive;


mod models;
mod controllers;
mod conn;

use std::env;
use hyper::header::{ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use iron::prelude::*;
use iron::status;
use router::Router;
use dotenv::dotenv;

static INDEX: &'static [u8] = include_bytes!("index.html");
fn index_handler(_: &mut Request) -> IronResult<Response> {
    let mut response = Response::with((status::Ok, INDEX));
    response.headers.set(
        ContentType(Mime(TopLevel::Text, SubLevel::Html,
                         vec![(Attr::Charset, Value::Utf8)]))
    );
    Ok(response)
}

fn main() {
    dotenv().ok();
    conn::establish_connection();
    env_logger::init().unwrap();

    let mut router: Router = Router::new();
    router.get("/", index_handler, "index");
    router.post("/upload", controllers::upload::upload_handler, "upload");
    router.get("/cfg", controllers::get::get_config, "config");
    router.get("/get_all_configs", controllers::get::get_all_configs, "get_all_configs");

    let port: u16 = env::var("PORT").unwrap_or("".to_string()).parse().unwrap_or(3000);
    let _server = Iron::new(router).http(("0.0.0.0", port)).unwrap();
    println!("Listening on port {}", port);
}
