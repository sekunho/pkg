#[derive(Clone)]
pub struct PasswordHash(pub(crate) String);

pub(crate) const DUMMY_USER_PASSWORD: &str = "abcd1234";

impl std::fmt::Debug for PasswordHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PasswordHash").field(&"[REDACTED]").finish()
    }
}

impl std::fmt::Display for PasswordHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl From<PasswordHash> for String {
    fn from(val: PasswordHash) -> Self {
        val.0
    }
}

impl AsRef<str> for PasswordHash {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "rusqlite")]
impl rusqlite::types::FromSql for PasswordHash {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let value = value.as_str()?;
        Ok(PasswordHash(value.to_string()))
    }
}

#[cfg(feature = "rusqlite")]
impl rusqlite::types::ToSql for PasswordHash {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::from(self.0.clone()))
    }
}

pub struct Password(pub(crate) String);

impl Password {
    #[cfg(test)]
    pub(crate) fn dummy() -> Password {
        Password("hunter2".to_string())
    }
}

impl From<String> for Password {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<Password> for String {
    fn from(val: Password) -> Self {
        val.0
    }
}

impl<'t> From<&'t Password> for &'t [u8] {
    fn from(val: &'t Password) -> Self {
        val.0.as_bytes()
    }
}

impl From<Password> for Vec<u8> {
    fn from(val: Password) -> Self {
        val.0.into()
    }
}

impl std::fmt::Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Password").field(&"[REDACTED]").finish()
    }
}

impl std::fmt::Display for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}
