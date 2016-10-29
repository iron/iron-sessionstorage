use std::sync::Arc;
use std::collections::HashMap;

use cookie;
use iron;
use iron::prelude::*;

use RawSession;
use SessionBackend;

enum CookieOrString {
    Cookie(cookie::Cookie),
    String(String)
}

pub struct SignedCookieSession {
    values: HashMap<String, CookieOrString>,
    signing_key: Arc<Vec<u8>>,
    cookie_modifier: Option<Arc<Box<Fn(cookie::Cookie) -> cookie::Cookie + Send + Sync>>>
}

impl RawSession for SignedCookieSession {
    fn get_raw(&self, key: &str) -> Option<&str> {
        match self.values.get(key) {
            Some(&CookieOrString::Cookie(ref x)) => Some(&x.value),
            Some(&CookieOrString::String(ref x)) => Some(x),
            None => None
        }
    }

    fn set_raw(&mut self, key: &str, value: String) {
        self.values.insert(
            key.to_owned(),
            CookieOrString::String(value)
        );
    }

    fn write(&self, res: &mut Response) {
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
                    if let Some(ref modifier) = self.cookie_modifier {
                        c = modifier(c);
                    }
                    c
                }
            });
        }
        res.headers.set(iron::headers::SetCookie(cookiejar.delta()));
    }
}

#[derive(Clone)]
pub struct SignedCookieBackend {
    signing_key: Arc<Vec<u8>>,
    cookie_modifier: Option<Arc<Box<Fn(cookie::Cookie) -> cookie::Cookie + Send + Sync + 'static>>>
}

impl SignedCookieBackend {
    pub fn new(signing_key: Vec<u8>) -> Self {
        SignedCookieBackend {
            signing_key: Arc::new(signing_key),
            cookie_modifier: None,
        }
    }

    pub fn set_cookie_modifier<F: Fn(cookie::Cookie) -> cookie::Cookie + Send + Sync + 'static>(&mut self, f: F) {
        self.cookie_modifier = Some(Arc::new(Box::new(f)));
    }
}

impl SessionBackend for SignedCookieBackend {
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
            cookie_modifier: self.cookie_modifier.clone(),
        }
    }

}
