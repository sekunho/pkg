use axum_core::extract::{FromRequestParts, OptionalFromRequestParts};
use axum_extra::extract::CookieJar;
use http::request::Parts;

use crate::{
    AuthnBackend,
    auth::AuthHandle,
    error::{AuthFailure, AuthRejection, AuthSessionError},
    session::SessionToken,
    user::CurrentUser,
};

impl<S, Backend> FromRequestParts<S> for AuthHandle<Backend>
where
    S: Send + Sync,
    Backend: AuthnBackend + Send + Sync + 'static,
{
    type Rejection = AuthSessionError;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> {
        std::future::ready(
            parts
                .extensions
                .get::<AuthHandle<Backend>>()
                .cloned()
                .ok_or(AuthSessionError::MissingLayer),
        )
    }
}

impl<S, Backend: AuthnBackend> FromRequestParts<S> for CurrentUser<Backend>
where
    S: Send + Sync,
    Backend: AuthnBackend + Send + Sync + 'static,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .expect("infallible");

        let session = AuthHandle::<Backend>::from_request_parts(parts, state).await?;

        let token = jar
            .get(session.session_name())
            .ok_or(AuthRejection::no_session())
            .and_then(|cookie| {
                SessionToken::try_from(cookie).map_err(|_err| AuthRejection::invalid_session())
            })?;

        let user = session
            .get_user(&token)
            .await
            .map_err(|_| AuthRejection::internal())?;

        match user {
            Some(user) => Ok(user).map(|u| CurrentUser(u)),
            None => Err(AuthRejection::no_session()),
        }
    }
}

impl<S, Backend: AuthnBackend> OptionalFromRequestParts<S> for CurrentUser<Backend>
where
    S: Send + Sync,
    Backend: AuthnBackend + Send + Sync + 'static,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        let result =
            <CurrentUser<Backend> as FromRequestParts<S>>::from_request_parts(parts, state).await;
        match result {
            Ok(user) => Ok(Some(user)),
            Err(AuthRejection {
                failure:
                    AuthFailure::NoSession | AuthFailure::NoPermission | AuthFailure::InvalidSession,
                ..
            }) => Ok(None),
            Err(err) => Err(err),
        }
    }
}
