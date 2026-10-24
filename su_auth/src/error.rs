use std::fmt::Display;

use axum_core::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use http::StatusCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AuthFailure {
    NoSession,
    NoPermission,
    InvalidSession,
    Internal,
}

pub struct AuthRejection {
    pub failure: AuthFailure,
    pub jar: Option<CookieJar>,
}

impl AuthRejection {
    #[must_use]
    pub fn no_session() -> Self {
        Self {
            failure: AuthFailure::NoSession,
            jar: None,
        }
    }

    #[must_use]
    pub fn internal() -> Self {
        Self {
            failure: AuthFailure::Internal,
            jar: None,
        }
    }

    #[must_use]
    pub fn invalid_session() -> Self {
        Self {
            failure: AuthFailure::InvalidSession,
            jar: None,
        }
    }

    #[must_use]
    pub fn set_cookie_jar(mut self, jar: CookieJar) -> Self {
        self.jar = Some(jar);
        self
    }
}

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        let status = match self.failure {
            AuthFailure::NoSession | AuthFailure::InvalidSession => StatusCode::UNAUTHORIZED,
            AuthFailure::NoPermission => StatusCode::FORBIDDEN,
            AuthFailure::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let mut res = match self {
            Self { jar: Some(jar), .. } => (status, jar).into_response(),
            _ => (status).into_response(),
        };

        res.extensions_mut().insert(self.failure);
        res
    }
}

impl From<AuthSessionError> for AuthRejection {
    fn from(err: AuthSessionError) -> Self {
        match err {
            AuthSessionError::MissingLayer => AuthRejection {
                failure: AuthFailure::Internal,
                jar: None,
            },
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum AuthSessionError {
    MissingLayer,
}

impl IntoResponse for AuthSessionError {
    fn into_response(self) -> axum_core::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
    }
}

impl Display for AuthSessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthSessionError::MissingLayer => {
                write!(f, "Failed to extract auth session. Missing layer")
            }
        }
    }
}

impl std::error::Error for AuthSessionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
