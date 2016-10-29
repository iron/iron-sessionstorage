extern crate cookie;
extern crate iron;

use iron::prelude::*;
use iron::middleware::{AroundMiddleware,Handler};
use iron::typemap;

pub mod backends;

pub trait Session {
    fn get(&self, key: &str) -> Option<&str>;
    fn set(&mut self, key: &str, value: String);
    fn write_cookies(&self, response: &mut Response);
}

pub trait SessionBackend: Send + Sync + Clone + 'static {
    type S: Session;
    fn from_request(&self, request: &mut Request) -> Self::S;
}


pub struct SessionStorage<B: SessionBackend> {
    backend: B
}

impl<B: SessionBackend> SessionStorage<B> {
    pub fn new(backend: B) -> Self {
        SessionStorage {
            backend: backend
        }
    }
}

type RequestSession = Box<Session>;

impl typemap::Key for RequestSession { type Value = RequestSession; }

impl<B: SessionBackend> AroundMiddleware for SessionStorage<B> {
    fn around(self, handler: Box<Handler>) -> Box<Handler> {
        Box::new(move |req: &mut Request| -> IronResult<Response> {
            let s = self.backend.from_request(req);
            req.extensions.insert::<RequestSession>(Box::new(s));
            let mut res = handler.handle(req);
            let s = req.extensions.remove::<RequestSession>().unwrap();
            match res {
                Ok(ref mut x) => s.write_cookies(x),
                Err(ref mut e) => s.write_cookies(&mut e.response)
            };
            res
        })
    }
}

pub trait RequestExt {
    fn session(&mut self) -> &mut RequestSession;
}

impl<'a, 'b> RequestExt for Request<'a, 'b> {
    fn session(&mut self) -> &mut RequestSession {
        self.extensions.get_mut::<RequestSession>().unwrap()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
