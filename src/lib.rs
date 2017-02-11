extern crate cookie as _cookie;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate iron;
extern crate rand;
#[cfg(feature = "redis-backend")] extern crate redis;
#[cfg(feature = "redis-backend")] extern crate r2d2;
#[cfg(feature = "redis-backend")] extern crate r2d2_redis;

use iron::prelude::*;
use iron::middleware::{AroundMiddleware,Handler};
use iron::typemap;

pub mod backends;
pub mod errors;

/// Re-export of the cookie crate
pub mod cookie {
    pub use _cookie::*;
}

/// A simple key-value storage interface that is internally used by `Session`. After request
/// handling the `write` method is called where the session backend has the chance to e.g. set
/// cookies or otherwise modify the response.
pub trait RawSession {
    fn get_raw(&self, key: &str) -> IronResult<Option<String>>;
    fn set_raw(&mut self, key: &str, value: String) -> IronResult<()>;
    fn clear(&mut self) -> IronResult<()>;
    fn write(&self, response: &mut Response) -> IronResult<()>;
}

pub trait SessionBackend: Send + Sync + 'static {
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
    inner: Box<RawSession>,
    has_changed: bool,
}

impl Session {
    fn new(s: Box<RawSession>) -> Self {
        Session {
            inner: s,
            has_changed: false
        }
    }
}


/// A typed interface to the string-to-string mapping. Each type represents a key, each instance of
/// a type can be serialized into a value.
pub trait Value: Sized + 'static {
    fn get_key() -> &'static str;
    fn into_raw(self) -> String;
    fn from_raw(value: String) -> Option<Self>;
}

impl Session {
    /// Get a `Value` from the session.
    pub fn get<T: Value + Sized + 'static>(&self) -> IronResult<Option<T>> {
        Ok(try!(self.inner.get_raw(T::get_key())).and_then(T::from_raw))
    }

    /// Set a `Value` in the session.
    pub fn set<T: Value>(&mut self, t: T) -> IronResult<()> {
        // FIXME: Equality check for less unnecessary writes
        self.has_changed = true;
        self.inner.set_raw(T::get_key(), t.into_raw())
    }

    /// Clear/delete the session
    pub fn clear(&mut self) -> IronResult<()> {
        // FIXME: Equality check for less unnecessary writes
        self.has_changed = true;
        self.inner.clear()
    }
}

struct SessionKey;
impl typemap::Key for SessionKey { type Value = Session; }

impl<B: SessionBackend> AroundMiddleware for SessionStorage<B> {
    fn around(self, handler: Box<Handler>) -> Box<Handler> {
        Box::new(move |req: &mut Request| -> IronResult<Response> {
            let s = self.backend.from_request(req);
            req.extensions.insert::<SessionKey>(Session::new(Box::new(s)));
            let mut res = handler.handle(req);
            let s = req.extensions.remove::<SessionKey>().unwrap();
            if s.has_changed {
                match res {
                    Ok(ref mut r) => try!(s.inner.write(r)),
                    Err(ref mut e) => try!(s.inner.write(&mut e.response))
                }
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
