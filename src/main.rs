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

mod models;
mod controllers;
mod conn;

use std::io::prelude::*;
use std::fs::File;
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

fn handler(req: &mut Request) -> IronResult<Response> {
    let ref config = req.extensions.get::<Router>().unwrap().find("config").unwrap();

    match File::open("./configs/".to_string() + config) {
        Ok(ref mut f) => {
            let mut contents = String::new();
            match f.read_to_string(&mut contents) {
                Ok(_) => Ok(Response::with((status::Ok, contents))),
                Err(_) => Ok(Response::with((status::Ok, contents))),
            }
        },
        Err(_) => Ok(Response::with((status::NotFound, "Not Found"))),
    }
}

fn main() {
    dotenv().ok();
    conn::establish_connection();
    env_logger::init().unwrap();

    let mut router = Router::new();
    router.get("/", index_handler, "index");
    router.post("/cfg", controllers::upload::upload_handler, "upload");
    router.get("/cfg/:config", handler, "config");

    let port: u16 = env::var("PORT").unwrap_or("".to_string()).parse().unwrap_or(3000);
    let _server = Iron::new(router).http(("0.0.0.0", port)).unwrap();
    println!("Listening on port {}", port);
}
