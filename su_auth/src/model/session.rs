use axum_extra::extract::cookie::Cookie;
use base64::{DecodeError, Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rusqlite::{ToSql, types::ToSqlOutput};
use sha2::Digest as _;

use crate::model::user::{Password, PasswordHash, UserId};

#[derive(Clone, Copy, Debug)]
pub struct SessionTokenHash(pub(crate) [u8; 32]);

impl ToSql for SessionTokenHash {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.0.as_ref()))
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

pub struct Login {
    pub user_id: Option<UserId>,
    pub password: Password,

    // Just strictly for dummy use if the user fetch fails.
    pub dummy_password_hash: PasswordHash,
}
