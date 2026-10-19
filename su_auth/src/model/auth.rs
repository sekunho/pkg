use axum_core::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use http::StatusCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AuthFailure {
    NoSession,
    InvalidSession,
    Banned,
    NotBanned,
    NotVerified,
    NotUnverified,
    Internal,
}

pub struct AuthRejection {
    pub failure: AuthFailure,
    pub jar: CookieJar,
}

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        let status = match self.failure {
            AuthFailure::NoSession | AuthFailure::InvalidSession => StatusCode::UNAUTHORIZED,
            AuthFailure::NotVerified
            | AuthFailure::NotUnverified
            | AuthFailure::Banned
            | AuthFailure::NotBanned => StatusCode::FORBIDDEN,
            AuthFailure::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let mut res = (status, self.jar).into_response();
        res.extensions_mut().insert(self.failure);
        res
    }
}
