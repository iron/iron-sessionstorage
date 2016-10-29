extern crate iron;
extern crate iron_sessionstorage;

use iron::prelude::*;

use iron_sessionstorage::{RequestExt,SessionStorage};
use iron_sessionstorage::backends::SignedCookieBackend;

fn handler(req: &mut Request) -> IronResult<Response> {
    let value = req.session().get("foo").unwrap_or("").to_owned();
    req.session().set("foo", format!("{}a", value));
    Ok(Response::with(format!("Reload this page to add an a: {}\n\n \
                              Clear cookies to reset.", value)))
}

fn main() {
    let my_secret = b"verysecret".to_vec();
    let mut ch = Chain::new(handler);
    ch.link_around(SessionStorage::new(SignedCookieBackend::new(my_secret)));
    let _res = Iron::new(ch).http("localhost:8080");
    println!("Listening on 8080.");
}
