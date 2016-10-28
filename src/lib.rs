extern crate iron;

use iron::prelude::*;
use iron::middleware::{AroundMiddleware,Handler};
use iron::typemap;

pub trait Session {
    fn get(&self, key: &str) -> Option<&str>;
    fn set(&mut self, key: &str, value: String);
    fn write_cookies(&self, response: &mut Response);
}

pub trait SessionBackend: Send + Sync + Clone + 'static {
    type S: Session;
    fn from_request(&self, request: &mut Request) -> Self::S;
}

mod signedcookie {
    extern crate cookie;
    use std::sync::Arc;

    use std::collections::HashMap;
    use iron;
    use iron::prelude::*;

    enum CookieOrString {
        Cookie(cookie::Cookie),
        String(String)
    }

    pub struct SignedCookieSession {
        values: HashMap<String, CookieOrString>,
        signing_key: Arc<Vec<u8>>,
    }

    impl super::Session for SignedCookieSession {
        fn get(&self, key: &str) -> Option<&str> {
            match self.values.get(key) {
                Some(&CookieOrString::Cookie(ref x)) => Some(&x.value),
                Some(&CookieOrString::String(ref x)) => Some(x),
                None => None
            }
        }

        fn set(&mut self, key: &str, value: String) {
            self.values.insert(
                key.to_owned(),
                CookieOrString::String(value)
            );
        }

        fn write_cookies(&self, res: &mut Response) {
            debug_assert!(!res.headers.has::<iron::headers::SetCookie>());

            let cookiejar = cookie::CookieJar::new(&self.signing_key);
            for (key, value) in self.values.iter() {
                cookiejar.signed().add(match value {
                    &CookieOrString::Cookie(ref x) => x.clone(),
                    &CookieOrString::String(ref x) => {
                        let mut c = cookie::Cookie::new(
                            key.to_owned(),
                            x.to_owned()
                        );
                        c.httponly = true;
                        c.path = Some("/".to_owned());
                        c
                    }
                });
            }
            res.headers.set(iron::headers::SetCookie(cookiejar.delta()));
        }
    }

    #[derive(Clone, Debug)]
    pub struct SignedCookieBackend {
        signing_key: Arc<Vec<u8>>,
    }

    impl SignedCookieBackend {
        pub fn new(signing_key: Vec<u8>) -> Self {
            SignedCookieBackend {
                signing_key: Arc::new(signing_key)
            }
        }
    }

    impl super::SessionBackend for SignedCookieBackend {
        type S = SignedCookieSession;

        fn from_request(&self, req: &mut Request) -> Self::S {
            let jar = match req.headers.get::<iron::headers::Cookie>() {
                Some(cookies) => cookies.to_cookie_jar(&self.signing_key),
                None => cookie::CookieJar::new(&self.signing_key)
            };

            SignedCookieSession {
                values: jar.signed().iter()
                    .map(|c| (c.name.clone(), CookieOrString::Cookie(c)))
                    .collect(),
                signing_key: self.signing_key.clone(),
            }
        }

    }
}

pub use signedcookie::SignedCookieBackend;


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
