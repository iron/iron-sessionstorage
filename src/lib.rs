extern crate cookie;
extern crate iron;

use iron::prelude::*;
use iron::middleware::{AroundMiddleware,Handler};
use iron::typemap;

pub mod backends;

pub trait RawSession {
    fn get_raw(&self, key: &str) -> Option<&str>;
    fn set_raw(&mut self, key: &str, value: String);
    fn write(&self, response: &mut Response);
}

pub trait SessionBackend: Send + Sync + Clone + 'static {
    type S: RawSession;
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

pub trait Value: Sized + 'static {
    fn get_key() -> &'static str;
    fn into_raw(self) -> String;
    fn from_raw(value: &str) -> Option<Self>;
}

pub struct Session {
    inner: Box<RawSession>
}

impl Session {
    pub fn get<T: Value + Sized + 'static>(&self) -> Option<T> {
        self.inner.get_raw(T::get_key()).and_then(T::from_raw)
    }

    pub fn set<T: Value>(&mut self, t: T) {
        self.inner.set_raw(T::get_key(), t.into_raw());
    }
}

impl typemap::Key for Session { type Value = Session; }

impl<B: SessionBackend> AroundMiddleware for SessionStorage<B> {
    fn around(self, handler: Box<Handler>) -> Box<Handler> {
        Box::new(move |req: &mut Request| -> IronResult<Response> {
            let s = self.backend.from_request(req);
            req.extensions.insert::<Session>(Session {
                inner: Box::new(s)
            });
            let mut res = handler.handle(req);
            let s = req.extensions.remove::<Session>().unwrap();
            match res {
                Ok(ref mut x) => s.inner.write(x),
                Err(ref mut e) => s.inner.write(&mut e.response)
            };
            res
        })
    }
}

pub trait RequestExt {
    fn session(&mut self) -> &mut Session;
}

impl<'a, 'b> RequestExt for Request<'a, 'b> {
    fn session(&mut self) -> &mut Session {
        self.extensions.get_mut::<Session>().unwrap()
    }
}

pub mod traits {
    pub use super::{RequestExt};
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
