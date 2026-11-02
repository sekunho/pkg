use axum_extra::extract::cookie::{Cookie, SameSite};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use http::{HeaderMap, HeaderValue, header};

use crate::session::SessionToken;

#[must_use]
pub fn create_session_cookie(
    name: String,
    session_token: SessionToken,
    max_age: time::Duration,
) -> Cookie<'static> {
    let mut cookie = Cookie::build(name)
        .same_site(SameSite::Lax)
        .path("/")
        .secure(true)
        .max_age(max_age)
        .http_only(true)
        .build();
    cookie.set_value(URL_SAFE_NO_PAD.encode(session_token.0));
    cookie
}

#[must_use]
pub fn invalidate(name: &str) -> Cookie<'_> {
    Cookie::build(name)
        .removal()
        .path("/")
        .same_site(SameSite::Lax)
        .secure(true)
        .http_only(true)
        .build()
}

pub fn set_cookie_header(headers: &mut HeaderMap, cookie: Cookie<'_>) {
    if let v = cookie.encoded().to_string().as_str()
        && let Ok(v) = HeaderValue::from_str(v)
    {
        headers.append(header::SET_COOKIE, v);
    }
}
