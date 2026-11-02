use std::fmt::Debug;

#[cfg(feature = "cookie")]
use base64::{DecodeError, Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::Digest as _;

use crate::{AuthnBackend, user::UserId};

pub trait AuthSession<Backend: AuthnBackend>: Debug + Clone {
    fn token_hash(&self) -> SessionTokenHash;
    fn expires_in(&self) -> std::time::Duration;
    fn created_ago(&self) -> std::time::Duration;
    fn extend_expiration(&mut self, duration: std::time::Duration);
    fn user_id(&self) -> UserId<Backend>;
    fn new(user_id: UserId<Backend>, token_hash: SessionTokenHash) -> Self;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
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

#[derive(Clone, Eq, PartialEq)]
pub struct SessionToken(pub(crate) Vec<u8>);

impl SessionToken {
    #[must_use]
    pub fn new(bs: Vec<u8>) -> Self {
        Self(bs)
    }
}

impl Debug for SessionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SessionToken").field(&"[REDACTED]").finish()
    }
}

impl AsRef<[u8]> for SessionToken {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[cfg(feature = "cookie")]
impl<'t> TryFrom<&axum_extra::extract::cookie::Cookie<'t>> for SessionToken {
    type Error = DecodeError;

    fn try_from(cookie: &axum_extra::extract::cookie::Cookie<'t>) -> Result<Self, Self::Error> {
        let token = URL_SAFE_NO_PAD.decode(cookie.value())?;
        Ok(SessionToken(token))
    }
}
