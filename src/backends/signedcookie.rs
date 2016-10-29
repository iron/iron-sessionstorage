use std::sync::Arc;
use std::collections::HashMap;

use cookie;
use iron;
use iron::prelude::*;

use Session;
use SessionBackend;

enum CookieOrString {
    Cookie(cookie::Cookie),
    String(String)
}

pub struct SignedCookieSession {
    values: HashMap<String, CookieOrString>,
    signing_key: Arc<Vec<u8>>,
}

impl Session for SignedCookieSession {
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
        }
    }

}
