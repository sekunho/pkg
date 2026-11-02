use axum_core::extract::{FromRequestParts, OptionalFromRequestParts};
use http::request::Parts;

use crate::{
    AuthnBackend,
    auth::AuthHandle,
    error::{AuthFailure, AuthRejection, AuthSessionError},
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
        let session_handle = AuthHandle::<Backend>::from_request_parts(parts, state).await?;

        let session = session_handle
            .session_data
            .lock()
            .await
            .clone()
            .ok_or_else(|| AuthRejection::no_session())?;
        Ok(session.user)
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
