#[macro_use] extern crate iron;
extern crate iron_sessionstorage;
#[macro_use] extern crate router;
extern crate urlencoded;

use iron::prelude::*;
use iron::status;
use iron::modifiers::Redirect;

use iron_sessionstorage::traits::*;
use iron_sessionstorage::SessionStorage;
use iron_sessionstorage::backends::SignedCookieBackend;

use urlencoded::UrlEncodedBody;

struct Login {
    username: String
}

impl iron_sessionstorage::Value for Login {
    fn get_key() -> &'static str { "logged_in_user" }
    fn into_raw(self) -> String { self.username }
    fn from_raw(value: String) -> Option<Self> {
        if value.is_empty() {
            None
        } else {
            Some(Login { username: value })
        }
    }
}

fn login(req: &mut Request) -> IronResult<Response> {
    if try!(req.session().get::<Login>()).is_some() {
        // Already logged in
        return Ok(Response::with((status::Found, Redirect(url_for!(req, "greet")))));
    }

    Ok(Response::with((
        status::Ok,
        "text/html".parse::<iron::mime::Mime>().unwrap(),
        format!("This is an insecure demo, so which username do you want to log in as?<br/> \n\
        <form method=post> \n\
        <input type=text name=username> \n\
        <input type=submit> \n\
        </form>")
    )))
}

fn login_post(req: &mut Request) -> IronResult<Response> {
    let username = {
        let formdata = iexpect!(req.get_ref::<UrlEncodedBody>().ok());
        iexpect!(formdata.get("username"))[0].to_owned()
    };

    try!(req.session().set(Login { username: username }));
    Ok(Response::with((status::Found, Redirect(url_for!(req, "greet")))))
}

fn logout(req: &mut Request) -> IronResult<Response> {
    try!(req.session().clear());
    Ok(Response::with((status::Found, Redirect(url_for!(req, "greet")))))
}

fn greet(req: &mut Request) -> IronResult<Response> {
    let login = iexpect!(
        req.session().get::<Login>().ok().and_then(|x| x),
        (
            status::Unauthorized,
            "text/html".parse::<iron::mime::Mime>().unwrap(),
            "<a href=/login>Log in</a>"
        )
    );

    Ok(Response::with((
        status::Ok,
        "text/html".parse::<iron::mime::Mime>().unwrap(),
        format!("Hello, {}! <br/>\n\
        <form method=post action=/logout>\n\
        <input type=submit value='Log out' />\n\
        </form>", login.username)
    )))
}

fn main() {
    let router = router!(
        greet: get "/" => greet,
        login: get "/login" => login,
        login_post: post "/login" => login_post,
        logout: post "/logout" => logout,
    );

    let my_secret = b"verysecret".to_vec();
    let mut ch = Chain::new(router);
    ch.link_around(SessionStorage::new(SignedCookieBackend::new(my_secret)));
    let _res = Iron::new(ch).http("localhost:8080");
    println!("Listening on 8080.");
}
