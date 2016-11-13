extern crate iron;
extern crate router;

use iron::prelude::*;
use iron::status;
use router::Router;

static INDEX: &'static [u8] = include_bytes!("index.html");

fn main() {
    let mut router = Router::new();
    router.get("/", index_handler, "index");
    router.get("/:config", handler, "config");

    fn index_handler(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, INDEX)))
    }

    fn handler(req: &mut Request) -> IronResult<Response> {
        let ref config = req.extensions.get::<Router>().unwrap().find("config").unwrap();
        Ok(Response::with((status::Ok, format!("Hello World! {}", config))))
    }

    let _server = Iron::new(router).http("localhost:3000").unwrap();
    println!("Listening on port 3000");
}

// Local Variables:
// flycheck-rust-crate-type: "bin"
// End:
