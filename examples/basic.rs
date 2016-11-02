#[macro_use] extern crate iron;
extern crate iron_sessionstorage;

use iron::prelude::*;

use iron_sessionstorage::traits::*;
use iron_sessionstorage::SessionStorage;
use iron_sessionstorage::backends::SignedCookieBackend;

struct Aaa(String);

impl iron_sessionstorage::Value for Aaa {
    fn get_key() -> &'static str { "foo" }
    fn into_raw(self) -> String { self.0 }
    fn from_raw(value: String) -> Option<Self> {
        // Maybe validate that only 'a's are in the string
        Some(Aaa(value))
    }
}

fn handler(req: &mut Request) -> IronResult<Response> {
    let mut value = match try!(req.session().get::<Aaa>()) {
        Some(aaa) => aaa,
        None => Aaa("".to_owned())
    };

    let res = Ok(Response::with(
        format!("Reload this page to add an a: {}\n\n \
                 Clear cookies to reset.", &value.0)
    ));

    value.0.push('a');
    try!(req.session().set(value));
    res
}

fn main() {
    let my_secret = b"verysecret".to_vec();
    let mut ch = Chain::new(handler);
    ch.link_around(SessionStorage::new(SignedCookieBackend::new(my_secret)));
    let _res = Iron::new(ch).http("localhost:8080");
    println!("Listening on 8080.");
}
