extern crate iron;
extern crate router;
extern crate dotenv;

use std::io::prelude::*;
use std::fs::File;
use std::env;
use iron::prelude::*;
use iron::status;
use router::Router;
use dotenv::dotenv;

static INDEX: &'static [u8] = include_bytes!("index.html");
fn index_handler(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, INDEX)))
}

fn handler(req: &mut Request) -> IronResult<Response> {
    let ref config = req.extensions.get::<Router>().unwrap().find("config").unwrap();

    match File::open("./configs/".to_string() + config) {
        Ok(ref mut f) => {
            let mut contents = String::new();
            match f.read_to_string(&mut contents) {
                Ok(_) => return Ok(Response::with((status::Ok, contents))),
                Err(_) => return Ok(Response::with((status::Ok, contents))),
            }
        },
        Err(_) => return Ok(Response::with((status::NotFound, "Not Found"))),
    };
}

fn main() {
    dotenv().ok();

    let mut router = Router::new();
    router.get("/", index_handler, "index");
    router.get("/:config", handler, "config");

    let port: u16 = env::var("PORT").unwrap_or("".to_string()).parse().unwrap_or(3000);
    let _server = Iron::new(router).http(("localhost", port)).unwrap();
    println!("Listening on port {}", port);
}

// Local Variables:
// flycheck-rust-crate-type: "bin"
// End:
