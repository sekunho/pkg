use std::error::Error;
use std::fmt::Debug;

use crate::{
    session::{AuthSession, SessionToken, SessionTokenHash},
    user::{AuthUser, UserId},
};

#[cfg(feature = "argon2")]
pub mod argon2;

pub mod auth;

#[cfg(feature = "cookie")]
pub mod cookie;

pub mod error;

#[cfg(feature = "middleware")]
pub mod middleware;

pub mod password;
pub mod session;
pub mod user;

#[cfg(feature = "verification-code")]
pub mod verification;

pub mod extract;

pub trait AuthnBackend: Debug + Clone + Send + Sync {
    type User: AuthUser<Self>;
    type Credentials: Send + Sync;
    type Session: AuthSession<Self> + Send + Sync;

    #[cfg(feature = "verification-code")]
    type Verification: crate::verification::AuthVerification<Self>;

    type AuthenticateError: Error + Clone + Send + Sync + 'static;
    type GetUserError: Error + Debug + Clone + Send + Sync + 'static;

    type CreateSessionError: Error + Send + Sync + 'static;
    type UpdateSessionError: Error + Send + Sync + 'static;
    type DeleteSessionError: Error + Send + Sync + 'static;
    type ValidateSessionError: Error + Send + Sync + 'static;

    type CreateVerificationCodeError: Error + Send + Sync + 'static;
    type UpdateVerificationCodeError: Error + Send + Sync + 'static;
    type DeleteVerificationCodeError: Error + Send + Sync + 'static;

    fn authenticate(
        &self,
        credentials: Self::Credentials,
    ) -> impl Future<Output = Result<Self::User, Self::AuthenticateError>> + Send;

    fn get_user(
        &self,
        user_id: &UserId<Self>,
    ) -> impl Future<Output = Result<Option<Self::User>, Self::GetUserError>> + Send;

    fn validate_session(
        &self,
        token: SessionTokenHash,
    ) -> impl Future<Output = Result<(), Self::ValidateSessionError>> + Send;

    fn generate_session_token() -> SessionToken;

    fn create_session(
        &self,
        session: Self::Session,
    ) -> impl Future<Output = Result<Self::Session, Self::CreateSessionError>> + Send;

    fn delete_session(
        &self,
        user_id: &UserId<Self>,
        token: SessionTokenHash,
    ) -> impl Future<Output = Result<(), Self::DeleteSessionError>> + Send;

    fn refresh_session(
        &self,
        token: &SessionTokenHash,
    ) -> impl Future<Output = Result<Option<Self::Session>, Self::UpdateSessionError>> + Send;

    fn get_user_from_session(
        &self,
        token_hash: SessionTokenHash,
    ) -> impl Future<Output = Result<Option<Self::User>, Self::GetUserError>> + Send;

    #[cfg(feature = "verification-code")]
    fn upsert_verification_code(
        &self,
        verification: Self::Verification,
    ) -> impl Future<Output = Result<Self::Verification, Self::CreateSessionError>> + Send;

    #[cfg(feature = "verification-code")]
    fn delete_expired_verification(
        &self,
    ) -> impl Future<Output = Result<(), Self::DeleteSessionError>> + Send;
}
