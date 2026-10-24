use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

use crate::session::SessionToken;

#[must_use]
pub fn create_session_cookie(name: &'static str, session_token: SessionToken) -> Cookie<'static> {
    let mut cookie = Cookie::build(name)
        .same_site(SameSite::Lax)
        .path("/")
        .secure(true)
        .max_age(time::Duration::hours(24 * 30))
        .http_only(true)
        .build();
    cookie.set_value(URL_SAFE_NO_PAD.encode(session_token.0));
    cookie
}

pub fn invalidate(jar: CookieJar, name: &'static str) -> CookieJar {
    jar.remove(
        Cookie::build(name)
            .path("/")
            .same_site(SameSite::Lax)
            .secure(true)
            .http_only(true),
    )
}
