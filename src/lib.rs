extern crate cookie;
extern crate iron;

use iron::prelude::*;
use iron::middleware::{AroundMiddleware,Handler};
use iron::typemap;

pub mod backends;

/// A simple key-value storage interface that is internally used by `Session`. After request
/// handling the `write` method is called where the session backend has the chance to e.g. set
/// cookies or otherwise modify the response.
pub trait RawSession {
    fn get_raw(&self, key: &str) -> Option<&str>;
    fn set_raw(&mut self, key: &str, value: String);
    fn write(&self, response: &mut Response);
}

pub trait SessionBackend: Send + Sync + Clone + 'static {
    type S: RawSession;

    /// Parse the session before request handling and return the data in parsed form.
    fn from_request(&self, request: &mut Request) -> Self::S;
}


/// The most important type, the middleware you need to register.
///
/// First parameter is a session backend, which determines how your session data is actually
/// stored. Depending on whether you store your data clientside (signed cookies) or serverside
/// (e.g. Redis and a session key) you're going to have different security properties.
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

/// The high-level interface you use to modify session data. You obtain this object with
/// `request.session()`.
pub struct Session {
    inner: Box<RawSession>
}

/// A typed interface to the string-to-string mapping. Each type represents a key, each instance of
/// a type can be serialized into a value.
pub trait Value: Sized + 'static {
    fn get_key() -> &'static str;
    fn into_raw(self) -> String;
    fn from_raw(value: &str) -> Option<Self>;
}

impl Session {
    /// Get a `Value` from the session.
    pub fn get<T: Value + Sized + 'static>(&self) -> Option<T> {
        self.inner.get_raw(T::get_key()).and_then(T::from_raw)
    }

    /// Set a `Value` in the session.
    pub fn set<T: Value>(&mut self, t: T) {
        self.inner.set_raw(T::get_key(), t.into_raw());
    }
}

struct SessionKey;
impl typemap::Key for SessionKey { type Value = Session; }

impl<B: SessionBackend> AroundMiddleware for SessionStorage<B> {
    fn around(self, handler: Box<Handler>) -> Box<Handler> {
        Box::new(move |req: &mut Request| -> IronResult<Response> {
            let s = self.backend.from_request(req);
            req.extensions.insert::<SessionKey>(Session {
                inner: Box::new(s)
            });
            let mut res = handler.handle(req);
            let s = req.extensions.remove::<SessionKey>().unwrap();
            match res {
                Ok(ref mut x) => s.inner.write(x),
                Err(ref mut e) => s.inner.write(&mut e.response)
            };
            res
        })
    }
}

/// The helper trait to obtain your session data from a request.
pub trait SessionRequestExt {
    fn session(&mut self) -> &mut Session;
}

impl<'a, 'b> SessionRequestExt for Request<'a, 'b> {
    fn session(&mut self) -> &mut Session {
        self.extensions.get_mut::<SessionKey>().unwrap()
    }
}

/// A module with some important traits to star-import.
pub mod traits {
    pub use super::{SessionRequestExt};
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
