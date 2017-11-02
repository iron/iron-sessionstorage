use std::sync::Arc;

use cookie;
use iron;
use iron::prelude::*;

use RawSession;
use SessionBackend;
use get_default_cookie;

pub struct SignedCookieSession {
    unsigned_jar: cookie::CookieJar,
    signing_key: cookie::Key,
    cookie_modifier: Option<Arc<Box<Fn(cookie::Cookie) -> cookie::Cookie + Send + Sync>>>
}

impl SignedCookieSession {
    fn signed_jar<'a>(&'a mut self) -> cookie::SignedJar<'a> {
        self.unsigned_jar.signed(&self.signing_key)
    }
}

impl RawSession for SignedCookieSession {
    fn get_raw(&mut self, key: &str) -> IronResult<Option<String>> {
        Ok(self.signed_jar().get(key).map(|c| c.value().to_owned()))
    }

    fn set_raw(&mut self, key: &str, value: String) -> IronResult<()> {
        let mut c = get_default_cookie(key.to_owned(), value);
        if let Some(ref modifier) = self.cookie_modifier {
            c = modifier(c);
        }
        self.signed_jar().add(c);
        Ok(())
    }

    fn clear(&mut self) -> IronResult<()> {
        self.unsigned_jar.clear();
        Ok(())
    }

    fn write(&self, res: &mut Response) -> IronResult<()> {
        debug_assert!(!res.headers.has::<iron::headers::SetCookie>());
        res.headers.set(iron::headers::SetCookie(
            self.unsigned_jar
            .delta()
            .into_iter()
            .map(|c| format!("{}", c))
            .collect()
        ));
        Ok(())
    }
}


/// Use signed cookies as session storage. See
/// http://lucumr.pocoo.org/2013/11/17/my-favorite-database/ for an introduction to this concept.
///
/// You need to pass a random value to the constructor of `SignedCookieBackend`. When this value is
/// changed, all session data is lost. Never publish this value, everybody who has it can forge
/// sessions.
///
/// Note that whatever you write into your session is visible by the user (but not modifiable).
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

    fn from_request<'a, 'r>(&'a self, req: &'r mut Request) -> Self::S {
        let mut jar = cookie::CookieJar::new();
        if let Some(cookies) = req.headers.get::<iron::headers::Cookie>() {
            for cookie in cookies.iter() {
                if let Ok(cookie) = cookie::Cookie::<'r>::parse(cookie.as_str()) {
                    jar.add_original(cookie.into_owned());
                }
            }
        };

        SignedCookieSession {
            unsigned_jar: jar,
            signing_key: cookie::Key::from_master(&self.signing_key),
            cookie_modifier: self.cookie_modifier.clone(),
        }
    }

}
