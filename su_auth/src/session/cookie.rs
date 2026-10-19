use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

use crate::model::session::SessionToken;

pub const COOKIE_SESSION_KEY: &str = "__Host-session";

#[must_use]
pub fn create_session_cookie(session_token: SessionToken) -> Cookie<'static> {
    let mut cookie = Cookie::build(COOKIE_SESSION_KEY)
        .same_site(SameSite::Lax)
        .path("/")
        .secure(true)
        .max_age(time::Duration::hours(24 * 30))
        .http_only(true)
        .build();
    cookie.set_value(URL_SAFE_NO_PAD.encode(session_token.0));
    cookie
}

pub fn invalidate(jar: CookieJar) -> CookieJar {
    jar.remove(
        Cookie::build(COOKIE_SESSION_KEY)
            .path("/")
            .same_site(SameSite::Lax)
            .secure(true)
            .http_only(true),
    )
}
