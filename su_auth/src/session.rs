use std::fmt::Debug;

use axum_extra::extract::cookie::Cookie;
use base64::{DecodeError, Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::Digest as _;

use crate::{AuthnBackend, user::UserId};

pub trait AuthSession<Backend: AuthnBackend>: Debug + Clone {
    fn token_hash(&self) -> SessionTokenHash;
    fn user_id(&self) -> UserId<Backend>;
    fn new(user: &Backend::User) -> Self;
}

#[derive(Clone, Copy, Debug)]
pub struct SessionTokenHash(pub(crate) [u8; 32]);

#[cfg(feature = "rusqlite")]
impl rusqlite::types::ToSql for SessionTokenHash {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::from(self.0.as_ref()))
    }
}

impl From<SessionToken> for SessionTokenHash {
    fn from(token: SessionToken) -> Self {
        SessionTokenHash(sha2::Sha256::digest(token).into())
    }
}

impl From<&SessionToken> for SessionTokenHash {
    fn from(token: &SessionToken) -> Self {
        SessionTokenHash(sha2::Sha256::digest(token).into())
    }
}

impl AsRef<[u8; 32]> for SessionTokenHash {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

pub struct SessionToken(pub(crate) Vec<u8>);

impl AsRef<[u8]> for SessionToken {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<'t> TryFrom<&Cookie<'t>> for SessionToken {
    type Error = DecodeError;

    fn try_from(cookie: &Cookie<'t>) -> Result<Self, Self::Error> {
        let token = URL_SAFE_NO_PAD.decode(cookie.value())?;
        Ok(SessionToken(token))
    }
}
