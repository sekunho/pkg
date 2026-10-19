use std::sync::Arc;

use axum_core::extract::{FromRef, FromRequestParts, OptionalFromRequestParts};
use axum_extra::extract::CookieJar;
use http::request::Parts;

use crate::{
    model::{
        auth::{AuthFailure, AuthRejection},
        user::{BannedUser, UnverifiedUser, User, VerifiedUser},
    },
    session,
};

impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
    Arc<su_sqlite::handle::Handle>: FromRef<S>,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .expect("infallible");

        let db = Arc::<su_sqlite::handle::Handle>::from_ref(state);

        let token = jar
            .clone()
            .get(session::cookie::COOKIE_SESSION_KEY)
            .ok_or_else(|| AuthRejection {
                failure: AuthFailure::NoSession,
                jar: jar.clone(),
            })
            .and_then(|cookie| {
                crate::model::session::SessionToken::try_from(cookie).map_err(|_| {
                    let jar = session::cookie::invalidate(jar.clone());
                    AuthRejection {
                        failure: AuthFailure::InvalidSession,
                        jar,
                    }
                })
            })?;

        let user = session::service::validate_session(&db, token)
            .await
            .map_err(|_| AuthRejection {
                failure: AuthFailure::Internal,
                jar: jar.clone(),
            })?;

        match user {
            None | Some(User::Deleted(_)) => {
                let jar = session::cookie::invalidate(jar.clone());
                Err(AuthRejection {
                    failure: AuthFailure::InvalidSession,
                    jar,
                })
            }
            Some(user) => Ok(user),
        }
    }
}

impl<S> OptionalFromRequestParts<S> for User
where
    S: Send + Sync,
    Arc<su_sqlite::handle::Handle>: FromRef<S>,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        match <User as FromRequestParts<S>>::from_request_parts(parts, state).await {
            Ok(user) => Ok(Some(user)),
            Err(AuthRejection {
                failure: AuthFailure::NoSession | AuthFailure::InvalidSession,
                ..
            }) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

impl<S> FromRequestParts<S> for VerifiedUser
where
    S: Send + Sync,
    Arc<su_sqlite::handle::Handle>: FromRef<S>,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .expect("infallible");

        match <User as FromRequestParts<S>>::from_request_parts(parts, state).await? {
            User::Verified(user) => Ok(user),
            User::Banned(_) => Err(AuthRejection {
                failure: AuthFailure::Banned,
                jar,
            }),
            _ => Err(AuthRejection {
                failure: AuthFailure::NotVerified,
                jar,
            }),
        }
    }
}

impl<S> FromRequestParts<S> for UnverifiedUser
where
    S: Send + Sync,
    Arc<su_sqlite::handle::Handle>: FromRef<S>,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .expect("infallible");

        match <User as FromRequestParts<S>>::from_request_parts(parts, state).await? {
            User::Unverified(user) => Ok(user),
            User::Banned(_) => Err(AuthRejection {
                failure: AuthFailure::Banned,
                jar,
            }),
            _ => Err(AuthRejection {
                failure: AuthFailure::NotUnverified,
                jar,
            }),
        }
    }
}

impl<S> FromRequestParts<S> for BannedUser
where
    S: Send + Sync,
    Arc<su_sqlite::handle::Handle>: FromRef<S>,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .expect("infallible");

        match <User as FromRequestParts<S>>::from_request_parts(parts, state).await? {
            User::Banned(user) => Ok(user),
            _ => Err(AuthRejection {
                failure: AuthFailure::NotBanned,
                jar,
            }),
        }
    }
}
